use std::{any::Any, collections::{HashMap, HashSet, VecDeque}, rc::Rc};
use indexmap::IndexSet;
use slotmap::{SecondaryMap, SlotMap};
use crate::param::{AnyParameterMap, ParamRef, ParameterId};
use super::{animation::AnimationState, app_state::Task, binding::BindingState, effect::EffectState, memo::MemoState, signal::SignalState, Memo, MemoContext, NodeId, Signal, SignalContext, SignalCreator, SignalGetContext, WidgetId};

struct Node {
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check,
    /// Reactive value is invalid, parents have changed, value needs to be recomputed
    Dirty
}

enum NodeType {
    Signal(SignalState),
    Memo(MemoState),
    Effect(EffectState),
    Binding(BindingState),
	Animation(AnimationState)
}

pub struct Runtime {
    nodes: SlotMap<NodeId, Node>,
    pub(super) sources: SecondaryMap<NodeId, IndexSet<NodeId>>,
    pub(super) observers: SecondaryMap<NodeId, IndexSet<NodeId>>,
    pub(super) pending_tasks: VecDeque<Task>,
    pub(super) parameters: Rc<dyn AnyParameterMap>,
	parameter_subscriptions: HashMap<ParameterId, IndexSet<NodeId>>,
	parameter_dependencies: SecondaryMap<NodeId, IndexSet<ParameterId>>,
}

impl Runtime {
    pub fn new(parameter_map: Rc<dyn AnyParameterMap>) -> Self {
		let parameter_ids = parameter_map.parameter_ids();
    
        let mut this = Self { 
            nodes: Default::default(), 
            sources: Default::default(), 
            observers: Default::default(),
            pending_tasks: Default::default(),
            parameters: parameter_map,
			parameter_subscriptions: Default::default(),
			parameter_dependencies: Default::default(),
        };

		for parameter_id in parameter_ids {
			this.parameter_subscriptions.insert(parameter_id, IndexSet::new());
		}

		this
    }

    pub(super) fn create_binding_node(&mut self, source_id: NodeId, state: BindingState) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
        self.add_subscription(source_id, id);
        id
    }

    pub(super) fn create_animation_node(&mut self, source_id: NodeId, state: AnimationState) -> NodeId {
        let id = self.create_node(NodeType::Animation(state), NodeState::Check);
        self.add_subscription(source_id, id);
        id
    }

	pub(super) fn create_parameter_binding_node(&mut self, source_id: ParameterId, state: BindingState) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
        self.add_parameter_subscription(source_id, id);
        id
    }

    fn create_node(&mut self, node_type: NodeType, state: NodeState) -> NodeId {
        let node = Node { node_type, state };
        let id = self.nodes.insert(node);
        self.sources.insert(id, IndexSet::new());
        self.observers.insert(id, IndexSet::new());
		self.parameter_dependencies.insert(id, IndexSet::new());
        id
    }

    pub(crate) fn remove_node(&mut self, id: NodeId) {
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

        self.nodes.remove(id).expect("Missing node");
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

        let mut subscribers = std::mem::take(&mut self.sources[signal.id]);
        subscribers.iter().for_each(|subscriber| {
            self.notify(subscriber);
        });
        std::mem::swap(&mut subscribers, &mut self.sources[signal.id]);
    }

    fn propagate_node_update(&mut self, node_id: NodeId) {
        // Traverse the graph in depth-first order, marking direct children of the node 
        // as Dirty, and the grand-children as Check
        let mut stack = vec![];

        // Mark direct children as dirty
        {
            let mut observers = std::mem::take(&mut self.observers[node_id]);
            for &observer_id in observers.iter() {
                self.nodes[observer_id].state = NodeState::Dirty;
                stack.push(observer_id);
            }
            std::mem::swap(&mut observers, &mut self.observers[node_id]);
        }

        let mut current = 0;
        while current < stack.len() {
            let node_id = stack[current];
            for child in self.observers[node_id].iter() {
                
            }
        }
    }

    fn mark_observers_dirty(&mut self, node_id: NodeId) {
        let mut observers = std::mem::take(&mut self.observers[node_id]);
        for &observer_id in observers.iter() {
            self.nodes[observer_id].state = NodeState::Dirty;
        }
        std::mem::swap(&mut observers, &mut self.observers[node_id]);
    }

    fn update_if_necessary(&mut self, node_id: NodeId) {
        if self.nodes[node_id].state == NodeState::Check {
            for source_id in self.sources[node_id].clone() {
                self.update_if_necessary(source_id);
                if self.nodes[node_id].state == NodeState::Dirty {
                    break;
                }
            }   
        }

        if self.nodes[node_id].state == NodeState::Dirty {
            // Update
        }

        self.nodes[node_id].state = NodeState::Clean;
    }

	pub(super) fn add_parameter_subscription(&mut self, source_id: ParameterId, observer_id: NodeId) {
        self.parameter_subscriptions.get_mut(&source_id).unwrap().insert(observer_id);
        self.parameter_dependencies[observer_id].insert(source_id);
	}

	pub(super) fn notify_parameter_subscribers(&mut self, source_id: ParameterId) {
		let subscribers = self.parameter_subscriptions.remove(&source_id).unwrap();
        subscribers.iter().for_each(|subscriber| {
            self.notify(subscriber);
        });
        self.parameter_subscriptions.insert(source_id, subscribers);
	}

	pub(super) fn add_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.sources[observer_id].insert(source_id);
        self.observers[source_id].insert(observer_id);
	}

    pub(super) fn remove_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.sources[source_id].swap_remove(&observer_id);
        self.observers[observer_id].swap_remove(&source_id);
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
			NodeType::Animation(_) => {

			},
            NodeType::Memo(memo) => {
                /*let _new_value = {
                    let mut ctx = MemoContext { memo_id: *node_id, runtime: self };
                    (memo.f)(&mut ctx)
                };*/
            },
            NodeType::Signal(_) => unreachable!(),
        }
    }

    pub(super) fn take_tasks(&mut self) -> VecDeque<Task> {
        std::mem::take(&mut self.pending_tasks)
    }
}

impl SignalGetContext for Runtime {
    fn get_node_value_ref_untracked<'a>(&'a self, node_id: NodeId) -> &'a dyn Any {
        let node = self.nodes.get(node_id).expect("Node not found");
        match &node.node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!()
        }
    }

    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.get_node_value_ref_untracked(signal_id)
    }
	
	fn get_parameter_ref_untracked(&self, parameter_id: ParameterId) -> ParamRef {
		self.parameters.get_by_id(parameter_id).expect("Invalid parameter id").as_param_ref()
	}
	
	fn get_parameter_ref(&mut self, parameter_id: ParameterId) -> ParamRef {
		self.get_parameter_ref_untracked(parameter_id)
	}
}

impl SignalContext for Runtime {
    fn set_signal_value<T: Any>(&mut self, signal: &Signal<T>, value: T) {
        self.update_signal_value(signal, move |val| *val = value);
    }
}

impl SignalCreator for Runtime {
    fn create_signal_node(&mut self, state: SignalState) -> NodeId {
        self.create_node(NodeType::Signal(state), NodeState::Clean)
    }

    fn create_memo_node(&mut self, state: MemoState) -> NodeId {
        self.create_node(NodeType::Memo(state), NodeState::Check)
    }

    fn create_effect_node(&mut self, state: EffectState) -> NodeId {
        self.create_node(NodeType::Effect(state), NodeState::Clean)
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