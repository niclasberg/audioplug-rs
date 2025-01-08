use std::{collections::{HashMap, HashSet, VecDeque}, rc::Rc};
use indexmap::IndexSet;
use slotmap::{SecondaryMap, SlotMap};
use crate::param::{AnyParameterMap, ParamRef, ParameterId};
use super::{animation::AnimationState, app_state::Task, binding::BindingState, effect::EffectState, memo::MemoState, signal::{SignalContext, SignalState}, MemoContext, NodeId, ReactiveContext, SignalCreator};

pub struct Node {
    pub(super) node_type: NodeType,
    state: NodeState
}

pub struct PathSegment(usize);
pub struct Path(Vec<PathSegment>);
impl Path {
    pub const ROOT: Self = Self(Vec::new());
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean = 0,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check = 1,
    /// Reactive value is invalid, parents have changed, value needs to be recomputed
    Dirty = 2
}

pub(super) enum NodeType {
    TmpRemoved,
	Trigger,
    Signal(SignalState),
    Memo(MemoState),
    Effect(EffectState),
    Binding(BindingState),
	Animation(AnimationState)
}

pub struct SubscriberMap {
    pub(super) sources: SecondaryMap<NodeId, IndexSet<NodeId>>,
    pub(super) observers: SecondaryMap<NodeId, IndexSet<NodeId>>,
	parameter_subscriptions: HashMap<ParameterId, IndexSet<NodeId>>,
	parameter_dependencies: SecondaryMap<NodeId, IndexSet<ParameterId>>,
}

impl SubscriberMap {
    fn new(parameter_ids: &Vec<ParameterId>) -> Self {
        let mut parameter_subscriptions = HashMap::new();
		for &parameter_id in parameter_ids {
            parameter_subscriptions.insert(parameter_id, IndexSet::new());
		}

        Self {
            sources: Default::default(), 
            observers: Default::default(), 
            parameter_subscriptions, 
            parameter_dependencies: Default::default() 
        }
    }

    pub fn insert_node(&mut self, node_id: NodeId) {
        self.sources.insert(node_id, IndexSet::new());
        self.observers.insert(node_id, IndexSet::new());
		self.parameter_dependencies.insert(node_id, IndexSet::new());
    }

    pub fn remove_node(&mut self, id: NodeId) {
        // Remove the node's subscriptions to other nodes
        let observers = self.observers.remove(id).expect("Missing observers for node");
        for node_id in observers {
            self.sources[node_id].swap_remove(&id);
        }

        // Remove other nodes' subscriptions to this node
        let sources = self.sources.remove(id).expect("Missing sources for node");
        for node_id in sources {
            self.observers[node_id].swap_remove(&id);
        }

		// Remove parameter subcriptions
		let parameter_dependencies = self.parameter_dependencies.remove(id).expect("Missing parameter dependencies for node");
		for parameter_id in parameter_dependencies {
            self.parameter_subscriptions.get_mut(&parameter_id).expect("Missing parameter subscription").swap_remove(&id);
        }
    }

    pub fn clear_node_sources(&mut self, node_id: NodeId) {
        let sources = self.sources.get_mut(node_id).expect("Missing sources for node");
        for node_id in sources.drain(..) {
            self.observers[node_id].swap_remove(&node_id);
        }
    }

    pub fn add_parameter_subscription(&mut self, source_id: ParameterId, observer_id: NodeId) {
        self.parameter_subscriptions.get_mut(&source_id).unwrap().insert(observer_id);
        self.parameter_dependencies[observer_id].insert(source_id);
	}

	pub fn add_node_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.sources[observer_id].insert(source_id);
        self.observers[source_id].insert(observer_id);
	}

    pub fn remove_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.sources[source_id].swap_remove(&observer_id);
        self.observers[observer_id].swap_remove(&source_id);
    }
}

pub struct Runtime {
    nodes: SlotMap<NodeId, Node>,
    child_nodes: SecondaryMap<NodeId, Vec<NodeId>>,
    pub(super) subscriptions: SubscriberMap,
    pub(super) pending_tasks: VecDeque<Task>,
    pub(super) parameters: Rc<dyn AnyParameterMap>,
    scratch_buffer: VecDeque<NodeId>
}

impl Runtime {
    pub fn new(parameter_map: Rc<dyn AnyParameterMap>) -> Self {    
        Self { 
            nodes: Default::default(), 
            child_nodes: Default::default(),
            subscriptions: SubscriberMap::new(&parameter_map.parameter_ids()),
            pending_tasks: Default::default(),
            parameters: parameter_map,
            scratch_buffer: Default::default(),
        }
    }

    pub(super) fn create_binding_node(&mut self, source_id: NodeId, state: BindingState) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
        self.subscriptions.add_node_subscription(source_id, id);
        id
    }

    pub(super) fn create_animation_node(&mut self, source_id: NodeId, state: AnimationState) -> NodeId {
        let id = self.create_node(NodeType::Animation(state), NodeState::Check);
        self.subscriptions.add_node_subscription(source_id, id);
        id
    }

	pub(super) fn create_parameter_binding_node(&mut self, source_id: ParameterId, state: BindingState) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
        self.subscriptions.add_parameter_subscription(source_id, id);
        id
    }

    fn create_node(&mut self, node_type: NodeType, state: NodeState) -> NodeId {
        let node = Node { node_type, state };
        let id = self.nodes.insert(node);
        self.subscriptions.insert_node(id);
        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.subscriptions.remove_node(id);
        self.nodes.remove(id).expect("Missing node");
    }

    fn notify_source_changed(&mut self, mut nodes_to_notify: VecDeque<NodeId>) {
        let mut nodes_to_check = HashSet::new();
        
        {
            let direct_child_count = nodes_to_notify.len();
            let mut i = 0;
            while let Some(node_id) = nodes_to_notify.pop_front() {
                // Mark direct nodes as Dirty and grand-children as Check
                let new_state = if i < direct_child_count { NodeState::Dirty } else { NodeState::Check };
                let node = self.nodes.get_mut(node_id).expect("Node has been removed");
                if node.state < new_state {
                    node.state = new_state;
                    match &node.node_type {
                        NodeType::Effect(_) | NodeType::Binding(_) | NodeType::Animation(_) => {
                            nodes_to_check.insert(node_id);
                        },
                        _ => {}
                    }
                    nodes_to_notify.extend(self.subscriptions.observers[node_id].iter());
                }
                i += 1;
            }
        }

        // Swap back the scratch buffer. Saves us from having to reallocate
        std::mem::swap(&mut self.scratch_buffer, &mut nodes_to_notify);

        for node_id in nodes_to_check {
            self.update_if_necessary(node_id);
        }
    }

    fn update_if_necessary(&mut self, node_id: NodeId) {
        if self.nodes[node_id].state == NodeState::Clean {
            return;
        }

        if self.nodes[node_id].state == NodeState::Check {
            for source_id in self.subscriptions.sources[node_id].clone() {
                self.update_if_necessary(source_id);
                if self.nodes[node_id].state == NodeState::Dirty {
                    break;
                }
            }   
        }

        if self.nodes[node_id].state == NodeState::Dirty {
            let mut node_type = std::mem::replace(&mut self.nodes[node_id].node_type, NodeType::TmpRemoved);
            match &mut node_type {
                NodeType::Effect(effect) => {
                    // Clear the sources, they will be re-populated while running the effect function
                    self.subscriptions.clear_node_sources(node_id);
                    let task = Task::RunEffect { 
                        id: node_id, 
                        f: Rc::downgrade(&effect.f)
                    };
                    self.pending_tasks.push_back(task);
                },
                NodeType::Binding(BindingState { widget_id, f }) => {
                    let task = Task::UpdateBinding { 
                        widget_id: widget_id.clone(),
                        f: Rc::downgrade(&f),
                        node_id
                    };
                    self.pending_tasks.push_back(task);
                },
                NodeType::Animation(_) => {
    
                },
                NodeType::Memo(memo) => {
                    // Clear the sources, they will be re-populated while running the memo function
                    self.subscriptions.clear_node_sources(node_id);
                    let mut cx = MemoContext { memo_id: node_id, runtime: self };
                    if memo.eval(&mut cx) {
                        for &observer_id in self.subscriptions.observers[node_id].iter() {
                            self.nodes[observer_id].state = NodeState::Dirty;
                        }
                    }
                },
				NodeType::Trigger => panic!("Triggers cannot depend on other reactive nodes"),
                NodeType::Signal(_) => panic!("Signals cannot depend on other reactive nodes"),
                NodeType::TmpRemoved => panic!("Circular dependency?")
            }
            std::mem::swap(&mut self.nodes[node_id].node_type, &mut node_type);
        }

        self.nodes[node_id].state = NodeState::Clean;
    }

	pub(super) fn notify_parameter_subscribers(&mut self, source_id: ParameterId) {
        let mut nodes_to_notify = std::mem::take(&mut self.scratch_buffer);
        nodes_to_notify.clear();
        nodes_to_notify.extend(self.subscriptions.parameter_subscriptions.get_mut(&source_id).unwrap().iter());
        self.notify_source_changed(nodes_to_notify);
	}

    pub(super) fn mark_node_as_clean(&mut self, node_id: NodeId) {
        self.nodes[node_id].state = NodeState::Clean;
    }

    pub(super) fn take_tasks(&mut self) -> VecDeque<Task> {
        std::mem::take(&mut self.pending_tasks)
    }
}

impl ReactiveContext for Runtime {
    fn get_node_mut(&mut self, node_id: NodeId, child_path: Path) -> &mut Node {
		self.update_if_necessary(node_id);
        self.nodes.get_mut(node_id).expect("Node not found")
    }
	
	fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef {
		self.parameters.get_by_id(parameter_id).expect("Invalid parameter id").as_param_ref()
	}
}

impl SignalContext for Runtime {
	fn notify(&mut self, node_id: NodeId) {
		let mut observers = std::mem::take(&mut self.scratch_buffer);
        observers.clear();
        observers.extend(self.subscriptions.observers[node_id].iter());
        self.notify_source_changed(observers);
	}
}

impl SignalCreator for Runtime {
    fn create_signal_node(&mut self, state: SignalState) -> NodeId {
        self.create_node(NodeType::Signal(state), NodeState::Clean)
    }

    fn create_memo_node(&mut self, state: MemoState) -> NodeId {
        self.create_node(NodeType::Memo(state), NodeState::Dirty)
    }

    fn create_effect_node(&mut self, state: EffectState) -> NodeId {
        let f = Rc::downgrade(&state.f);
        let id = self.create_node(NodeType::Effect(state), NodeState::Dirty);
        self.pending_tasks.push_back(Task::RunEffect { id, f });
        id
    }

	fn create_trigger(&mut self) -> NodeId {
		self.create_node(NodeType::Trigger, NodeState::Clean)
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