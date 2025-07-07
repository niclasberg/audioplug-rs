use super::{
    accessor::SourceId,
    animation::{AnimationState, DerivedAnimationState},
    app_state::Task,
    effect::{BindingState, EffectState},
    event_channel::EventHandlerState,
    memo::MemoState,
    signal::SignalState,
    NodeId, ReactiveContext, ReadContext, WidgetId, WindowId, WriteContext,
};
use crate::{
    app::FxHashMap,
    param::{AnyParameterMap, ParamRef, ParameterId},
};
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use slotmap::{SecondaryMap, SlotMap};
use std::{
    any::Any,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

pub struct Node {
    pub(super) node_type: NodeType,
    state: NodeState,
}

#[derive(Debug, Clone, Copy)]
pub enum Owner {
    Widget(WidgetId),
    Node(NodeId),
}

#[derive(Debug, Clone, Copy)]
pub enum Scope {
    Root,
    Node(NodeId),
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
    Dirty = 2,
}

pub enum NodeType {
    TmpRemoved,
    Trigger,
    Signal(SignalState),
    Memo(MemoState),
    Effect(EffectState),
    Binding(BindingState),
    Animation(AnimationState),
    DerivedAnimation(DerivedAnimationState),
    EventEmitter,
    EventHandler(EventHandlerState),
}

impl NodeType {
    pub fn get_value_ref(&self) -> &dyn Any {
        match self {
            NodeType::TmpRemoved => {
                panic!("Trying to get the value of a node that is currently removed")
            }
            NodeType::Trigger => &(),
            NodeType::Signal(signal_state) => signal_state.value.as_ref(),
            NodeType::Memo(memo_state) => memo_state
                .value
                .as_ref()
                .expect("Memo should have been evaluated before accessed")
                .as_ref(),
            NodeType::Effect(_) => panic!("Cannot get value of an effect"),
            NodeType::Binding(_) => panic!("Cannot get value of a binding"),
            NodeType::Animation(state) => state.inner.value_dyn(),
            NodeType::DerivedAnimation(state) => state.inner.value_dyn(),
            NodeType::EventEmitter => panic!("Cannot get value of eventemitter"),
            NodeType::EventHandler(_) => {
                panic!("Cannot get value of EventHandler")
            }
        }
    }
}

pub struct SubscriberMap {
    pub(super) sources: SecondaryMap<NodeId, IndexSet<NodeId>>,
    pub(super) observers: SecondaryMap<NodeId, IndexSet<NodeId>>,
    parameter_subscriptions: FxHashMap<ParameterId, IndexSet<NodeId>>,
    parameter_dependencies: SecondaryMap<NodeId, IndexSet<ParameterId>>,
}

impl SubscriberMap {
    fn new(parameter_ids: &Vec<ParameterId>) -> Self {
        let mut parameter_subscriptions = HashMap::with_hasher(FxBuildHasher::new());
        for &parameter_id in parameter_ids {
            parameter_subscriptions.insert(parameter_id, IndexSet::new());
        }

        Self {
            sources: Default::default(),
            observers: Default::default(),
            parameter_subscriptions,
            parameter_dependencies: Default::default(),
        }
    }

    pub fn insert_node(&mut self, node_id: NodeId) {
        self.sources.insert(node_id, IndexSet::new());
        self.observers.insert(node_id, IndexSet::new());
        self.parameter_dependencies.insert(node_id, IndexSet::new());
    }

    pub fn remove_node(&mut self, id: NodeId) {
        // Remove the node's subscriptions to other nodes
        let observers = self
            .observers
            .remove(id)
            .expect("Missing observers for node");
        for node_id in observers {
            self.sources[node_id].swap_remove(&id);
        }

        // Remove other nodes' subscriptions to this node
        let sources = self.sources.remove(id).expect("Missing sources for node");
        for node_id in sources {
            self.observers[node_id].swap_remove(&id);
        }

        // Remove parameter subcriptions
        let parameter_dependencies = self
            .parameter_dependencies
            .remove(id)
            .expect("Missing parameter dependencies for node");
        for parameter_id in parameter_dependencies {
            self.parameter_subscriptions
                .get_mut(&parameter_id)
                .expect("Missing parameter subscription")
                .swap_remove(&id);
        }
    }

    pub fn clear_node_sources(&mut self, node_id: NodeId) {
        let sources = self
            .sources
            .get_mut(node_id)
            .expect("Missing sources for node");
        for node_id in sources.drain(..) {
            self.observers[node_id].swap_remove(&node_id);
        }
    }

    pub fn add_parameter_subscription(&mut self, source_id: ParameterId, observer_id: NodeId) {
        self.parameter_subscriptions
            .get_mut(&source_id)
            .unwrap()
            .insert(observer_id);
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
    pub(super) subscriptions: SubscriberMap,
    pub(super) pending_tasks: VecDeque<Task>,
    pub(super) parameters: Rc<dyn AnyParameterMap>,
    scratch_buffer: VecDeque<NodeId>,
    nodes_owned_by_node: SecondaryMap<NodeId, HashSet<NodeId>>,
    nodes_owned_by_widget: SecondaryMap<WidgetId, HashSet<NodeId>>,
}

impl Runtime {
    pub fn new(parameter_map: Rc<dyn AnyParameterMap>) -> Self {
        Self {
            nodes: Default::default(),
            subscriptions: SubscriberMap::new(&parameter_map.parameter_ids()),
            pending_tasks: Default::default(),
            parameters: parameter_map,
            scratch_buffer: Default::default(),
            nodes_owned_by_node: Default::default(),
            nodes_owned_by_widget: Default::default(),
        }
    }

    pub(crate) fn create_signal_node(
        &mut self,
        state: SignalState,
        owner: Option<Owner>,
    ) -> NodeId {
        self.create_node(NodeType::Signal(state), NodeState::Clean, owner)
    }

    pub(crate) fn create_memo_node(&mut self, state: MemoState, owner: Option<Owner>) -> NodeId {
        self.create_node(NodeType::Memo(state), NodeState::Dirty, owner)
    }

    pub(crate) fn create_effect_node(
        &mut self,
        state: EffectState,
        owner: Option<Owner>,
        run_effect: bool,
    ) -> NodeId {
        let f = Rc::downgrade(&state.f);
        let id = self.create_node(NodeType::Effect(state), NodeState::Dirty, owner);
        if run_effect {
            self.pending_tasks.push_back(Task::RunEffect { id, f });
        }
        id
    }

    pub(crate) fn create_trigger(&mut self, owner: Option<Owner>) -> NodeId {
        self.create_node(NodeType::Trigger, NodeState::Clean, owner)
    }

    pub(crate) fn create_binding_node(
        &mut self,
        source: SourceId,
        state: BindingState,
        owner: Option<Owner>,
    ) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean, owner);
        match source {
            SourceId::Parameter(source_id) => {
                self.subscriptions.add_parameter_subscription(source_id, id)
            }
            SourceId::Node(source_id) => self.subscriptions.add_node_subscription(source_id, id),
        };
        id
    }

    pub(crate) fn create_animation_node(
        &mut self,
        state: AnimationState,
        owner: Option<Owner>,
    ) -> NodeId {
        self.create_node(NodeType::Animation(state), NodeState::Clean, owner)
    }

    pub(crate) fn create_derived_animation_node(
        &mut self,
        state_fn: impl FnOnce(&mut Self, NodeId) -> DerivedAnimationState,
        owner: Option<Owner>,
    ) -> NodeId {
        let id = self.create_node(NodeType::TmpRemoved, NodeState::Clean, owner);
        let state = state_fn(self, id);
        let _ = std::mem::replace(
            &mut self.nodes[id].node_type,
            NodeType::DerivedAnimation(state),
        );
        id
    }

    pub(crate) fn create_event_emitter(&mut self, owner: Option<Owner>) -> NodeId {
        self.create_node(NodeType::EventEmitter, NodeState::Clean, owner)
    }

    fn create_node(
        &mut self,
        node_type: NodeType,
        state: NodeState,
        owner: Option<Owner>,
    ) -> NodeId {
        let node = Node { node_type, state };
        let id = self.nodes.insert(node);
        self.subscriptions.insert_node(id);

        match owner {
            Some(Owner::Widget(widget_id)) => {
                self.nodes_owned_by_widget
                    .entry(widget_id)
                    .unwrap()
                    .or_default()
                    .insert(id);
            }
            Some(Owner::Node(node_id)) => {
                self.nodes_owned_by_node
                    .entry(node_id)
                    .unwrap()
                    .or_default()
                    .insert(id);
            }
            _ => {}
        };

        id
    }

    pub(super) fn clear_nodes_for_widget(&mut self, widget_id: WidgetId) {
        if let Some(bindings) = self.nodes_owned_by_widget.remove(widget_id) {
            for node_id in bindings {
                self.remove_node(node_id);
            }
        }
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.subscriptions.remove_node(id);
        self.nodes.remove(id).expect("Missing node");
        if let Some(child_ids) = self.nodes_owned_by_node.remove(id) {
            for child_id in child_ids {
                self.remove_node(child_id);
            }
        }
    }

    pub fn get_node(&self, node_id: NodeId) -> &Node {
        self.nodes.get(node_id).expect("Node not found")
    }

    pub fn try_get_node(&self, node_id: NodeId) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node {
        self.nodes.get_mut(node_id).expect("Node not found")
    }

    pub fn get_node_value_ref(&self, node_id: NodeId) -> Option<&dyn Any> {
        self.nodes
            .get(node_id)
            .map(|node| NodeType::get_value_ref(&node.node_type))
    }

    pub fn try_get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    pub fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef {
        self.parameters
            .get_by_id(parameter_id)
            .expect("Invalid parameter id")
            .as_param_ref()
    }

    /// Temporarily remove a node and return it
    pub(super) fn lease_node(&mut self, node_id: NodeId) -> Option<NodeType> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            Some(std::mem::replace(&mut node.node_type, NodeType::TmpRemoved))
        } else {
            None
        }
    }

    /// Return a node that has previously been leased
    pub(super) fn unlease_node(&mut self, node_id: NodeId, mut node: NodeType) {
        std::mem::swap(&mut self.nodes[node_id].node_type, &mut node);
    }

    pub fn notify(&mut self, node_id: NodeId) {
        let mut observers = std::mem::take(&mut self.scratch_buffer);
        observers.clear();
        observers.extend(self.subscriptions.observers[node_id].iter());
        self.notify_source_changed(observers);
    }

    fn notify_source_changed(&mut self, mut nodes_to_notify: VecDeque<NodeId>) {
        let mut nodes_to_check = HashSet::new();

        {
            let direct_child_count = nodes_to_notify.len();
            let mut i = 0;
            while let Some(node_id) = nodes_to_notify.pop_front() {
                // Mark direct nodes as Dirty and grand-children as Check
                let new_state = if i < direct_child_count {
                    NodeState::Dirty
                } else {
                    NodeState::Check
                };
                let node = self.nodes.get_mut(node_id).expect("Node has been removed");
                if node.state < new_state {
                    node.state = new_state;
                    match &node.node_type {
                        NodeType::Effect(_)
                        | NodeType::Binding(_)
                        | NodeType::DerivedAnimation(_) => {
                            nodes_to_check.insert(node_id);
                        }
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

    pub fn update_if_necessary(&mut self, node_id: NodeId) {
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
            let mut node_type =
                std::mem::replace(&mut self.nodes[node_id].node_type, NodeType::TmpRemoved);
            match &mut node_type {
                NodeType::Effect(EffectState { f }) => {
                    // Clear the sources, they will be re-populated while running the effect function
                    self.subscriptions.clear_node_sources(node_id);
                    let task = Task::RunEffect {
                        id: node_id,
                        f: Rc::downgrade(f),
                    };
                    self.pending_tasks.push_back(task);
                }
                NodeType::Binding(BindingState { f }) => {
                    let task = Task::UpdateBinding {
                        f: Rc::downgrade(f),
                        node_id,
                    };
                    self.pending_tasks.push_back(task);
                }
                NodeType::DerivedAnimation(anim) => {
                    // Clear the sources, they will be re-populated while running the reset function
                    self.subscriptions.clear_node_sources(node_id);
                    if anim.reset(node_id, self) {
                        self.request_animation(anim.window_id, node_id);
                    }
                }
                NodeType::Memo(memo) => {
                    // Clear the sources, they will be re-populated while running the memo function
                    self.subscriptions.clear_node_sources(node_id);
                    if memo.eval(node_id, self) {
                        for &observer_id in self.subscriptions.observers[node_id].iter() {
                            self.nodes[observer_id].state = NodeState::Dirty;
                        }
                    }
                }
                NodeType::Animation(..) => {
                    panic!("Animations cannot depend on other reactive nodes")
                }
                NodeType::Trigger => panic!("Triggers cannot depend on other reactive nodes"),
                NodeType::Signal(_) => panic!("Signals cannot depend on other reactive nodes"),
                NodeType::EventEmitter => {
                    panic!("Event emitters cannot depend on other reactive nodes")
                }
                NodeType::EventHandler(..) => {
                    panic!("Event handlers should not be notified, use publish_event instead")
                }
                NodeType::TmpRemoved => panic!("Circular dependency?"),
            }
            std::mem::swap(&mut self.nodes[node_id].node_type, &mut node_type);
        }

        self.nodes[node_id].state = NodeState::Clean;
    }

    pub(super) fn notify_parameter_subscribers(&mut self, source_id: ParameterId) {
        let mut nodes_to_notify = std::mem::take(&mut self.scratch_buffer);
        nodes_to_notify.clear();
        nodes_to_notify.extend(
            self.subscriptions
                .parameter_subscriptions
                .get_mut(&source_id)
                .unwrap()
                .iter(),
        );
        self.notify_source_changed(nodes_to_notify);
    }

    pub(super) fn mark_node_as_clean(&mut self, node_id: NodeId) {
        self.nodes[node_id].state = NodeState::Clean;
    }

    pub(super) fn take_tasks(&mut self) -> VecDeque<Task> {
        std::mem::take(&mut self.pending_tasks)
    }

    pub fn publish_event(&mut self, source_id: NodeId, event: Rc<dyn Any>) {}

    pub(super) fn request_animation(&mut self, window_id: WindowId, node_id: NodeId) {
        let task = Task::UpdateAnimation { node_id, window_id };
        self.pending_tasks.push_back(task);
    }

    pub fn track(&mut self, source_id: NodeId, scope: Scope) {
        if let Scope::Node(node_id) = scope {
            self.subscriptions.add_node_subscription(source_id, node_id);
        }
    }

    pub fn track_parameter(&mut self, source_id: crate::param::ParameterId, scope: Scope) {
        if let Scope::Node(node_id) = scope {
            self.subscriptions
                .add_parameter_subscription(source_id, node_id);
        }
    }
}

impl ReactiveContext for Runtime {
    fn runtime(&self) -> &Runtime {
        self
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self
    }
}

impl WriteContext for Runtime {}

impl ReadContext for Runtime {
    fn scope(&self) -> Scope {
        Scope::Root
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
