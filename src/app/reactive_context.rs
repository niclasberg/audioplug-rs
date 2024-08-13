use std::{any::Any, collections::{HashSet, VecDeque}, rc::Rc};

use slotmap::{SecondaryMap, SlotMap};

use super::{app_state::Task, binding::BindingState, effect::EffectState, memo::MemoState, signal::SignalState, Memo, NodeId, Signal, SignalContext};




pub struct Node {
    pub(super) node_type: NodeType,
    pub(super) state: NodeState
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

pub(super) enum NodeState {
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
pub(super) enum Scope {
    Root,
    Memo(NodeId),
    Effect(NodeId)
}


pub struct ReactiveContext {
    pub(super) scope: Scope,
    pub(super) nodes: SlotMap<NodeId, Node>,
    pub(super) subscriptions: SecondaryMap<NodeId, HashSet<NodeId>>,
    pub(super) dependencies: SecondaryMap<NodeId, HashSet<NodeId>>,
    pending_tasks: VecDeque<Task>,
}

impl Default for ReactiveContext {
    fn default() -> Self {
        Self { 
            scope: Scope::Root, 
            nodes: Default::default(), 
            subscriptions: Default::default(), 
            dependencies: Default::default(),
            pending_tasks: Default::default()
        }
    }
}

impl ReactiveContext {
    pub(super) fn with_scope<R>(&mut self, scope: Scope, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_scope = self.scope;
        self.scope = scope;
        let value = f(self);
        self.scope = old_scope;
        value
    }

    pub fn create_signal_node(&mut self, state: SignalState) -> NodeId {
        self.create_node(NodeType::Signal(state), NodeState::Clean)
    }

    pub fn create_memo_node(&mut self, state: MemoState) -> NodeId {
        self.create_node(NodeType::Memo(state), NodeState::Check)
    }

    pub fn create_effect_node(&mut self, state: EffectState) -> NodeId {
        self.create_node(NodeType::Effect(state), NodeState::Clean)
    }

    pub fn create_binding_node(&mut self, source_id: NodeId, state: BindingState) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
        self.add_subscription(source_id, id);
        id
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

    fn get_memo_value_ref_untracked<'a, T: Any>(&'a self, memo: &Memo<T>) -> &'a T {
        todo!()
    }

    fn get_memo_value_ref<'a, T: Any>(&'a mut self, memo: &Memo<T>) -> &'a T {
        self.track(memo.id);
        self.get_memo_value_ref_untracked(memo)
    }

	fn add_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.subscriptions[source_id].insert(observer_id);
        self.dependencies[observer_id].insert(source_id);
	}

    fn remove_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.subscriptions[source_id].remove(&observer_id);
        self.dependencies[observer_id].remove(&source_id);
    }

    pub(super) fn notify(&mut self, node_id: &NodeId) {
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

    pub(super) fn take_tasks(&mut self) -> VecDeque<Task> {
        std::mem::take(&mut self.pending_tasks)
    }
}

impl SignalContext for ReactiveContext {
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