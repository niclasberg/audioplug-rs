use std::{any::Any, cell::RefCell, collections::{HashSet, VecDeque}, rc::{Rc, Weak}};
use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};
use crate::{core::Point, param::Params, platform::{self, HandleRef}, view::Widget, window::WindowState, IdPath, MouseEvent};

use super::{binding::BindingState, contexts::BuildContext, memo::{Memo, MemoState}, ref_count_map::RefCountMap, widget_node::{WidgetData, WidgetId, WidgetMut, WidgetNode, WidgetRef}, Accessor, Binding, RenderContext, SignalContext, SignalGet, WindowId};
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
                    /*root_widget.with_child(widget_id, |node| {
                        f(cx, node)
                    });*/
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

pub(super) struct Window {
    pub(super) handle: platform::Handle,
    pub(super) root_widget: WidgetId,
}

pub struct AppState {
    scope: Scope,
    pending_tasks: VecDeque<Task>,
    nodes: SlotMap<NodeId, Node>,
    subscriptions: SecondaryMap<NodeId, HashSet<NodeId>>,
    dependencies: SecondaryMap<NodeId, HashSet<NodeId>>,
    node_ref_counts: Rc<RefCell<RefCountMap>>,
    parameters: Box<dyn Any>,
    pub(super) widget_data: SlotMap<WidgetId, WidgetData>,
    pub(super) widgets: SecondaryMap<WidgetId, Box<dyn Widget>>,
    widget_bindings: SecondaryMap<NodeId, WidgetId>,
    windows: SlotMap<WindowId, Window>,
    pub(super) mouse_capture_widget: Option<WidgetId>,
    pub(super) focus_widget: Option<WidgetId>
}

impl AppState {
    pub fn new(parameters: impl Params + Any) -> Self {
        Self {
            scope: Scope::Root,
            pending_tasks: Default::default(),
            nodes: Default::default(),
            subscriptions: Default::default(),
            dependencies: Default::default(),
            node_ref_counts: Rc::new(RefCell::new(RefCountMap::new())),
            parameters: Box::new(parameters),
            widget_data: Default::default(),
            widgets: Default::default(),
            widget_bindings: Default::default(),
            windows: Default::default(),
            mouse_capture_widget: None,
            focus_widget: None
        }
    }

    pub(crate) fn parameters_as<P: Params>(&self) -> Option<&P> {
        self.parameters.downcast_ref()
    }

    pub(crate) fn parameters_as_mut<P: Params>(&mut self) -> Option<&mut P> {
        self.parameters.downcast_mut()
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

    pub fn create_binding<T: 'static>(&mut self, accessor: Accessor<T>, widget_id: WidgetId, f: impl Fn(&T, &mut WidgetNode) + 'static) -> Option<Binding> {
        if let Some(source_id) = accessor.get_source_id() {
            /*let state = BindingState::new(widget_id, move |app_state, node| {
                accessor.with_ref(app_state, |value| {
                    f(value, node);
                });
            });
            let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
            self.add_subscription(source_id, id);
            let binding = Binding::new(id, Rc::downgrade(&self.node_ref_counts));
            Some(binding)*/
            todo!()
        } else {
            None
        }
    }

    pub fn create_effect(&mut self, f: impl Fn(&mut AppState) + 'static) {
        let id = self.create_node(NodeType::Effect(EffectState::new(f)), NodeState::Clean);
        self.notify(&id);
    }

    pub fn add_window<W: Widget>(&mut self, handle: platform::Handle, f: impl FnOnce(&mut BuildContext) -> W) -> WindowId {
        let root_widget = self.add_widget(WidgetId::null(), f);
        let window = Window {
            handle,
            root_widget
        };
        self.windows.insert(window)
    }

    pub fn remove_window(&mut self, id: WindowId) {
        let window = self.windows.remove(id).expect("Window not found");
        self.remove_widget(window.root_widget);
    }

    /// Add a new widget
    pub fn add_widget<W: Widget>(&mut self, parent_id: WidgetId, f: impl FnOnce(&mut BuildContext) -> W) -> WidgetId {
        let id = if parent_id.is_null() {
            self.widget_data.insert_with_key(|id| {
                WidgetData::new(id, id)
            })
        } else {
            let root_id = self.widget_data.get(parent_id).expect("Parent not found").window_id;
            self.widget_data.insert_with_key(|id| {
                WidgetData::new(id, root_id)
            })
        };
        
        {
            let widget = f(&mut BuildContext { id, app_state: self });
            self.widgets.insert(id, Box::new(widget));
        }

        if !parent_id.is_null() {
            let parent_widget_data = self.widget_data.get_mut(parent_id).expect("Parent does not exist");
            parent_widget_data.children.push(id);
        }

        id
    }

    /// Remove a widget and all of its children and associated signals
    pub fn remove_widget(&mut self, id: WidgetId) {
        let mut widget_data = self.widget_data.remove(id).unwrap();
        self.widgets.remove(id).expect("Widget already removed");

        // Must be removed from the children of the parent
        if !widget_data.parent_id.is_null() {
            let parent_id = widget_data.parent_id;
            let parent_widget_data = self.widget_data.get_mut(parent_id).expect("Parent does not exist");
            parent_widget_data.children.retain(|id| *id != parent_id);
        }

        let mut children_to_remove = std::mem::take(&mut widget_data.children);
        while let Some(id) = children_to_remove.pop() {
            let mut widget_data = self.widget_data.remove(id).unwrap();
            self.widgets.remove(id).expect("Widget already removed");

            children_to_remove.extend(widget_data.children.into_iter());
        }
    }

    pub fn widget_ref(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(id, self)
    }

    pub fn widget_mut(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(id, self)
    }

    pub fn widget_has_focus(&self, id: WidgetId) -> bool {
        self.focus_widget.as_ref()
            .is_some_and(|focus_widget_id| *focus_widget_id == id)
    }

    pub fn widget_data_ref(&self, id: WidgetId) -> &WidgetData {
        self.widget_data.get(id).expect("Widget data not found")
    }

    pub fn widget_data_mut(&mut self, id: WidgetId) -> &mut WidgetData {
        self.widget_data.get_mut(id).expect("Widget data not found")
    }

    /// Calls `f` for each widget that conatains `pos`. The order is from the root and down the tree (depth first order)
    pub fn for_each_widget_at(&self, id: WindowId, pos: Point, mut f: impl FnMut(&WidgetData) -> bool) {
        let mut stack = vec![self.windows[id].root_widget];
        while let Some(current) = stack.pop() {
            let data = &self.widget_data[current];
            if !f(data) {
                return;
            }

            for child in self.widget_data[current].children.iter().rev() {
                if data.global_bounds().contains(pos) {
                    stack.push(*child)
                }
            }
        }
    }

    pub fn for_each_widget_at_rev(&self, id: WindowId, pos: Point, mut f: impl FnMut(&WidgetData) -> bool) {
        let mut widget_id = self.windows[id].root_widget;
        todo!()
    }

    pub fn window(&self, id: WindowId) -> &Window {
        self.windows.get(id).expect("Window handle not found")
    }

    pub fn get_window_id_for_widget(&self, widget_id: WidgetId) -> WindowId {

    }

    pub fn mouse_event(&mut self, root_widget_id: WidgetId, event: MouseEvent) {
        
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
				/*let task = Task::UpdateBinding { 
                    widget_id: widget_id.clone(),
                    f: Rc::downgrade(&f)
                };
				self.pending_tasks.push_back(task);*/
                todo!()
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