use std::{any::Any, collections::{HashMap, HashSet, VecDeque}, rc::Rc};

use slotmap::{SecondaryMap, SlotMap};

use crate::param::{AnyParameter, AnyParameterMap, Parameter, ParameterId, ParameterMap, Params};

use super::{app_state::Task, binding::BindingState, effect::EffectState, memo::MemoState, param::ParamSignal, signal::SignalState, Memo, NodeId, Signal, SignalContext};

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
    Binding(BindingState)
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Scope {
    Root,
    Memo(NodeId),
    Effect(NodeId)
}

pub struct Runtime {
    pub(super) scope: Scope,
    nodes: SlotMap<NodeId, Node>,
    pub(super) subscriptions: SecondaryMap<NodeId, HashSet<NodeId>>,
    pub(super) dependencies: SecondaryMap<NodeId, HashSet<NodeId>>,
    pub(super) pending_tasks: VecDeque<Task>,
    pub(super) parameters: Box<dyn AnyParameterMap>,
	parameter_subscriptions: HashMap<ParameterId, HashSet<NodeId>>,
	parameter_dependencies: SecondaryMap<NodeId, HashSet<ParameterId>>,
}

impl Runtime {
    pub fn new(parameters: impl Params + Any) -> Self {
		let parameter_map = ParameterMap::new(parameters);
		let parameter_ids: Vec<ParameterId> = parameter_map.iter()
			.map(|param_ref| param_ref.id())
			.collect();

		let parameters = Box::new(parameter_map);
        let mut this = Self { 
            scope: Scope::Root, 
            nodes: Default::default(), 
            subscriptions: Default::default(), 
            dependencies: Default::default(),
            pending_tasks: Default::default(),
            parameters,
			parameter_subscriptions: Default::default(),
			parameter_dependencies: Default::default(),
        };

		for parameter_id in parameter_ids {
			this.parameter_subscriptions.insert(parameter_id, HashSet::new());
		}

		this
    }

    pub(super) fn with_scope<R>(&mut self, scope: Scope, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_scope = self.scope;
        self.scope = scope;
        let value = f(self);
        self.scope = old_scope;
        value
    }

    pub(super) fn create_signal_node(&mut self, state: SignalState) -> NodeId {
        self.create_node(NodeType::Signal(state), NodeState::Clean)
    }

    pub(super) fn create_memo_node(&mut self, state: MemoState) -> NodeId {
        self.create_node(NodeType::Memo(state), NodeState::Check)
    }

    pub(super) fn create_effect_node(&mut self, state: EffectState) -> NodeId {
        self.create_node(NodeType::Effect(state), NodeState::Clean)
    }

    pub(super) fn create_binding_node(&mut self, source_id: NodeId, state: BindingState) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean);
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
        self.subscriptions.insert(id, HashSet::new());
        self.dependencies.insert(id, HashSet::new());
		self.parameter_dependencies.insert(id, HashSet::new());
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

		// Remove parameter subcriptions
		let parameter_dependencies = self.parameter_dependencies.remove(id).expect("Missing parameter dependencies for node");
		for parameter_id in parameter_dependencies {
            self.parameter_subscriptions.get_mut(&parameter_id).expect("Missing parameter subscription").remove(&id);
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

	fn track_parameter(&mut self, parameter_id: ParameterId) {

	}

    fn get_memo_value_ref_untracked<'a, T: Any>(&'a self, memo: &Memo<T>) -> &'a T {
        todo!()
    }

    fn get_memo_value_ref<'a, T: Any>(&'a mut self, memo: &Memo<T>) -> &'a T {
        self.track(memo.id);
        self.get_memo_value_ref_untracked(memo)
    }

	fn add_parameter_subscription(&mut self, source_id: ParameterId, observer_id: NodeId) {
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

impl SignalContext for Runtime {
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

    fn get_parameter_value<T: Any>(&mut self, parameter: &ParamSignal<T>) -> T {
        self.get_parameter_value_untracked(parameter)
    }
    
    fn get_parameter_value_untracked<T: Any>(&self, parameter: &ParamSignal<T>) -> T {
        if let Some(p) = self.parameters.get_by_id(parameter.id) {
			p.value_as().unwrap()
        } else {
            unreachable!()
        }
    }
}