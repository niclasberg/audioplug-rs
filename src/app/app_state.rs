use std::{any::Any, cell::RefCell, collections::{HashSet, VecDeque}, rc::{Rc, Weak}};
use slotmap::{SlotMap, SecondaryMap};
use crate::{platform::HandleRef, view::{ViewMessage, Widget, WidgetContext, WidgetNode}, window::{Window, WindowState}, App, IdPath};

use super::{binding::BindingState, memo::{Memo, MemoState}, ref_count_map::RefCountMap, Accessor, Binding, SignalContext, SignalGet};
use super::NodeId;
use super::signal::{Signal, SignalState};
use super::effect::EffectState;

enum Task {
    RunEffect {
        id: NodeId,
        f: Weak<Box<dyn Fn(&mut AppState)>>
    },
	UpdateBinding {
		widget_id: IdPath,
    	f: Weak<Box<dyn Fn(&mut AppState, &mut WidgetNode)>>,
	}
}

impl Task {
    fn run(&self, cx: &mut AppState, root_widget: &mut WidgetNode, window_state: &mut WindowState, handle: &mut HandleRef) {
        match self {
            Task::RunEffect { id, f } => {
                cx.with_scope(Scope::Effect(*id), |cx| {
                    if let Some(f) = f.upgrade() {
                        f(cx)
                    }
                })
            },
            Task::UpdateBinding { widget_id, f } => {
                if let Some(f) = f.upgrade() {
                    root_widget.with_child(widget_id, |node| {
                        f(cx, node)
                    });
                }
            }
        }
    }
}

pub struct Node {
    node_type: NodeType,
    state: NodeState
}

/*impl Node {
    fn get_value_as<T: Any>(&self) -> Option<&T> {
        match &self.node_type {
            NodeType::Signal(signal) => signal.value.downcast_ref(),
            NodeType::Memo(memo) => memo.value.as_ref().and_then(|value| value.downcast_ref()),
            NodeType::Effect(_) => None,
        }
    }
}*/

enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check,
    /// Reactive value is invalid, parents have changed, value needs to be recomputed
    Dirty
}

pub enum NodeType {
    Signal(SignalState),
    Memo(MemoState),
    Effect(EffectState),
    Binding(BindingState)
}

#[derive(Debug, Clone, Copy)]
enum Scope {
    Root,
    Memo(NodeId),
    Effect(NodeId)
}

pub struct AppState {
    scope: Scope,
    pending_tasks: VecDeque<Task>,
    nodes: SlotMap<NodeId, Node>,
    subscriptions: SecondaryMap<NodeId, HashSet<NodeId>>,
    dependencies: SecondaryMap<NodeId, HashSet<NodeId>>,
    node_ref_counts: Rc<RefCell<RefCountMap>>
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scope: Scope::Root,
            pending_tasks: Default::default(),
            nodes: Default::default(),
            subscriptions: Default::default(),
            dependencies: Default::default(),
            node_ref_counts: Rc::new(RefCell::new(RefCountMap::new()))
        }
    }

    pub fn create_signal<T: Any>(&mut self, value: T) -> Signal<T> {
        let state = SignalState::new(value);
        let id = self.create_node(NodeType::Signal(state), NodeState::Clean);
        Signal::new(id, Rc::downgrade(&self.node_ref_counts))
    }

    pub fn create_memo<T: PartialEq + 'static>(&mut self, f: impl Fn(&mut Self) -> T + 'static) -> Memo<T> {
        let state = MemoState::new(move |cx| Box::new(f(cx)));
        let id = self.create_node(NodeType::Memo(state), NodeState::Check);
        Memo::new(id, Rc::downgrade(&self.node_ref_counts))
    }

    pub fn create_binding<T: 'static>(&mut self, accessor: Accessor<T>, widget_id: IdPath, f: impl Fn(&T, &mut WidgetNode) + 'static) -> Option<Binding> {
        if let Some(source_id) = accessor.get_source_id() {
            let state = BindingState::new(widget_id, move |app_state, node| {
                accessor.with_ref(app_state, |value| {
                    f(value, node);
                });
            });
            let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
            self.add_subscription(source_id, id);
            let binding = Binding::new(id, Rc::downgrade(&self.node_ref_counts));
            Some(binding)
        } else {
            None
        }
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut AppState) + 'static) {
        let id = self.create_node(NodeType::Effect(EffectState::new(f)), NodeState::Clean);
        self.notify(&id);
    }

    fn with_scope<R>(&mut self, scope: Scope, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_scope = self.scope;
        self.scope = scope;
        let value = f(self);
        self.scope = old_scope;
        value
    }

    fn create_node(&mut self, node_type: NodeType, state: NodeState) -> NodeId {
        let node = Node { node_type, state };
        let id = self.nodes.insert(node);
        self.subscriptions.insert(id, HashSet::new());
        self.dependencies.insert(id, HashSet::new());
        id
    }

    fn remove_node(&mut self, id: NodeId) -> Node {
        // Remove the node's subscriptions to other nodes
        let node_dependencies = self.dependencies.remove(id).expect("Missing dependencies for node");
        for node_id in node_dependencies {
            self.subscriptions[node_id].remove(&id);
        }

        // Remove other nodes' subscriptions to this node
        let node_subscriptions = self.subscriptions.remove(id).expect("Missing subscriptions for node");
        for node_id in node_subscriptions {
            self.dependencies[node_id].remove(&id);
        }

        self.nodes.remove(id).expect("Missing node")
    }

    fn update_signal_value<T: Any>(&mut self, signal: &Signal<T>, f: impl FnOnce(&mut T)) {
        {
            let signal = self.nodes.get_mut(signal.id).expect("No signal found");
            match &mut signal.node_type {
                NodeType::Signal(signal) => {
                    let mut value = signal.value.downcast_mut().expect("Invalid signal value type");
                    f(&mut value);
                },
                _ => unreachable!()
            }
            signal.state = NodeState::Dirty;
        }

        let mut subscribers = std::mem::take(&mut self.subscriptions[signal.id]);
        subscribers.iter().for_each(|subscriber| {
            self.notify(subscriber);
        });
        std::mem::swap(&mut subscribers, &mut self.subscriptions[signal.id]);
    }

    fn track(&mut self, source_id: NodeId) {
        match self.scope {
            Scope::Root => { 
                // Not in an effect/memo, nothing to track
            },
            Scope::Memo(node_id) | Scope::Effect(node_id) => {
                self.add_subscription(source_id, node_id);
            },
        }
    }

	fn add_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.subscriptions[source_id].insert(observer_id);
        self.dependencies[observer_id].insert(source_id);
	}

    fn remove_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.subscriptions[source_id].remove(&observer_id);
        self.dependencies[observer_id].remove(&source_id);
    }

    fn notify(&mut self, node_id: &NodeId) {
        let node = self.nodes.get_mut(*node_id).expect("Node has been removed");
        match &mut node.node_type {
            NodeType::Effect(effect) => {
                let task = Task::RunEffect { 
                    id: *node_id, 
                    f: Rc::downgrade(&effect.f)
                };
				self.pending_tasks.push_back(task);
            },
            NodeType::Binding(BindingState { widget_id, f }) => {
				let task = Task::UpdateBinding { 
                    widget_id: widget_id.clone(),
                    f: Rc::downgrade(&f)
                };
				self.pending_tasks.push_back(task);
            },
            NodeType::Memo(_) => todo!(),
            NodeType::Signal(_) => unreachable!(),
        }
    }

    fn get_memo_value_ref_untracked<'a, T: Any>(&'a self, memo: &Memo<T>) -> &'a T {
        todo!()
    }

    fn get_memo_value_ref<'a, T: Any>(&'a mut self, memo: &Memo<T>) -> &'a T {
        self.track(memo.id);
        self.get_memo_value_ref_untracked(memo)
    }

    /*pub fn create_stateful_effect<S>(&mut self, f_init: impl FnOnce() -> S, f: impl Fn(S) -> S) {

    }*/

    pub fn run_effects(&mut self, root_widget: &mut WidgetNode, window_state: &mut WindowState, handle: &mut HandleRef) {
        loop {
            if let Some(task) = self.pending_tasks.pop_front() {
                task.run(self, root_widget, window_state, handle)
            } else {
                break;
            }
        }
    }
}

impl SignalContext for AppState {
    fn get_signal_value_ref_untracked<'a, T: Any>(&'a self, signal: &Signal<T>) -> &'a T {
        let node = self.nodes.get(signal.id).expect("No Signal found");
        match &node.node_type {
            NodeType::Signal(signal) => signal.value.downcast_ref().expect("Node had wrong value type"),
            _ => unreachable!()
        }
    }

    fn get_signal_value_ref<'a, T: Any>(&'a mut self, signal: &Signal<T>) -> &'a T {
        self.track(signal.id);
        self.get_signal_value_ref_untracked(signal)
    }

    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.update_signal_value(signal, move |x| { *x = value });
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_signal() {
        let mut cx = ReactiveGraph::new();
        let signal = cx.create_signal(0);
        let mapped_signal = signal.clone().map(|x| x * 2);    
        signal.set(&mut cx, 2);
        cx.create_effect(move |cx| {
            assert_eq!(mapped_signal.get(cx), 4);
        });
        assert_eq!(mapped_signal.get_untracked(&cx), 4);
    }

    #[test]
    fn effects_execute_upon_creation() {
        let mut cx = ReactiveGraph::new();
        let signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            signal.set(cx, 1);
        });
        
        assert_eq!(signal.get_untracked(&cx), 1);
    }

    #[test]
    fn effects_execute_when_signal_changes() {
        let mut cx = ReactiveGraph::new();
        let source_signal = cx.create_signal(0);
        let dest_signal = cx.create_signal(0);
        cx.create_effect(move |cx| {
            let new_value = source_signal.get(cx);
            dest_signal.set(cx, new_value);
        });

        source_signal.set(&mut cx, 1);
        assert_eq!(dest_signal.get_untracked(&cx), 1);

        source_signal.set(&mut cx, 2);
        assert_eq!(dest_signal.get_untracked(&cx), 2);
    }
}*/