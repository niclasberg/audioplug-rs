use super::{
    NodeId,
    animation::{AnimationState, DerivedAnimationState},
    cached::CachedState,
    effect::{BindingState, EffectState},
    event_channel::EventHandlerState,
    var::SignalState,
};
use crate::{
    param::{AnyParameterMap, ParamRef, ParameterId},
    ui::{
        AppState, FxHashMap, FxHashSet, FxIndexSet, WidgetId, app_state::Task,
        widget_status::WidgetStatusFlags,
    },
};
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use std::{any::Any, collections::VecDeque, rc::Rc, time::Instant};

pub struct Node {
    pub(crate) node_type: NodeType,
    pub(crate) state: NodeState,
}

#[derive(Debug, Clone, Copy)]
pub enum Owner {
    Widget(WidgetId),
    Node(NodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SourceId {
    Parameter(ParameterId),
    Node(NodeId),
    Widget(WidgetId),
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
pub enum NodeState {
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
    Memo(CachedState),
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

    pub fn update(&mut self, node_id: NodeId, app_state: &mut AppState) {
        match self {
            NodeType::Effect(EffectState { f }) => {
                // Clear the sources, they will be re-populated while running the effect function
                app_state.runtime.subscriptions.clear_node_sources(node_id);
                let task = Task::RunEffect {
                    id: node_id,
                    f: Rc::downgrade(f),
                };
                app_state.push_task(task);
            }
            NodeType::Binding(BindingState { f }) => {
                let task = Task::UpdateBinding {
                    f: Rc::downgrade(f),
                    node_id,
                };
                app_state.push_task(task);
            }
            NodeType::DerivedAnimation(anim) => {
                // Clear the sources, they will be re-populated while running the reset function
                app_state.runtime.subscriptions.clear_node_sources(node_id);
                if anim.reset(node_id, self) {
                    app_state.request_animation(anim.window_id, node_id);
                }
            }
            NodeType::Memo(memo) => {
                // Clear the sources, they will be re-populated while running the memo function
                app_state.runtime.subscriptions.clear_node_sources(node_id);
                if memo.eval(node_id, self) {
                    for &observer_id in app_state.runtime.subscriptions.observers[node_id].iter() {
                        app_state.runtime.nodes[observer_id].state = NodeState::Dirty;
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
    }
}

pub struct SubscriberMap {
    sources: SecondaryMap<NodeId, SmallVec<[SourceId; 4]>>,
    observers: SecondaryMap<NodeId, FxIndexSet<NodeId>>,
    parameter_observers: FxHashMap<ParameterId, FxIndexSet<NodeId>>,
    widget_observers: SecondaryMap<WidgetId, SmallVec<[(NodeId, WidgetStatusFlags); 4]>>,
}

impl SubscriberMap {
    fn new(parameter_ids: &Vec<ParameterId>) -> Self {
        let mut parameter_subscriptions = FxHashMap::default();
        for &parameter_id in parameter_ids {
            parameter_subscriptions.insert(parameter_id, FxIndexSet::default());
        }

        Self {
            sources: Default::default(),
            observers: Default::default(),
            parameter_observers: parameter_subscriptions,
            widget_observers: Default::default(),
        }
    }

    pub fn insert_node(&mut self, node_id: NodeId) {
        self.observers.insert(node_id, FxIndexSet::default());
    }

    pub fn remove_node(&mut self, id: NodeId) {
        // Remove the node's subscriptions to other nodes
        let observers = self
            .observers
            .remove(id)
            .expect("Missing observers for node");
        for observer_id in observers {
            self.sources[observer_id].retain(|source_id| *source_id != SourceId::Node(id));
        }

        // Remove other nodes' subscriptions to this node
        if let Some(sources) = self.sources.remove(id) {
            for source_id in sources {
                Self::remove_source_from_observer(
                    &mut self.observers,
                    &mut self.parameter_observers,
                    &mut self.widget_observers,
                    source_id,
                    id,
                );
            }
        }
    }

    #[inline(always)]
    fn remove_source_from_observer(
        observers: &mut SecondaryMap<NodeId, FxIndexSet<NodeId>>,
        parameter_observers: &mut FxHashMap<ParameterId, FxIndexSet<NodeId>>,
        widget_observers: &mut SecondaryMap<WidgetId, SmallVec<[(NodeId, WidgetStatusFlags); 4]>>,
        source_id: SourceId,
        observer_id: NodeId,
    ) {
        match source_id {
            SourceId::Parameter(parameter_id) => {
                parameter_observers
                    .get_mut(&parameter_id)
                    .expect("Missing parameter subscription")
                    .swap_remove(&observer_id);
            }
            SourceId::Node(node_id) => {
                observers[node_id].swap_remove(&observer_id);
            }
            SourceId::Widget(widget_id) => {
                widget_observers[widget_id].retain(|(node_id, _)| *node_id != observer_id);
            }
        }
    }

    pub fn clear_node_sources(&mut self, node_id: NodeId) {
        if let Some(sources) = self.sources.get_mut(node_id) {
            for source_id in sources.drain(..) {
                Self::remove_source_from_observer(
                    &mut self.observers,
                    &mut self.parameter_observers,
                    &mut self.widget_observers,
                    source_id,
                    node_id,
                );
            }
        }
    }

    pub fn add_parameter_subscription(&mut self, source_id: ParameterId, observer_id: NodeId) {
        self.parameter_observers
            .get_mut(&source_id)
            .unwrap()
            .insert(observer_id);
        self.sources
            .entry(observer_id)
            .unwrap()
            .or_default()
            .push(SourceId::Parameter(source_id));
    }

    pub fn add_node_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        self.observers[source_id].insert(observer_id);
        self.sources
            .entry(observer_id)
            .unwrap()
            .or_default()
            .push(SourceId::Node(source_id));
    }

    pub fn add_widget_status_subscription(
        &mut self,
        widget_id: WidgetId,
        status_mask: WidgetStatusFlags,
        observer_id: NodeId,
    ) {
        // Maybe check if it exists and then merge masks?
        let observers = self.widget_observers.entry(widget_id).unwrap().or_default();
        observers.push((observer_id, status_mask));
        self.sources
            .entry(observer_id)
            .unwrap()
            .or_default()
            .push(SourceId::Widget(widget_id));
    }
}

pub struct ReactiveGraph {
    nodes: SlotMap<NodeId, Node>,
    pub(crate) subscriptions: SubscriberMap,
    pub(crate) parameters: Rc<dyn AnyParameterMap>,
    nodes_owned_by_node: SecondaryMap<NodeId, FxHashSet<NodeId>>,
    nodes_owned_by_widget: SecondaryMap<WidgetId, FxHashSet<NodeId>>,
}

impl ReactiveGraph {
    pub fn new(parameter_map: Rc<dyn AnyParameterMap>) -> Self {
        Self {
            nodes: Default::default(),
            subscriptions: SubscriberMap::new(&parameter_map.parameter_ids()),
            parameters: parameter_map,
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

    pub(crate) fn create_memo_node(&mut self, state: CachedState, owner: Option<Owner>) -> NodeId {
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

    pub(crate) fn create_node_binding_node(
        &mut self,
        source: NodeId,
        state: BindingState,
        owner: Option<Owner>,
    ) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean, owner);
        self.subscriptions.add_node_subscription(source, id);
        id
    }

    pub(crate) fn create_parameter_binding_node(
        &mut self,
        source: ParameterId,
        state: BindingState,
        owner: Option<Owner>,
    ) -> NodeId {
        let id = self.create_node(NodeType::Binding(state), NodeState::Clean, owner);
        self.subscriptions.add_parameter_subscription(source, id);
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

    pub(crate) fn clear_nodes_for_widget(&mut self, widget_id: WidgetId) {
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

    pub fn try_drive_animation(&mut self, node_id: NodeId, now: Instant) -> bool {
        if let Some(node) = self.nodes.get_mut(node_id) {
            match &mut node.node_type {
                NodeType::Animation(animation) => animation.inner.drive(now),
                NodeType::DerivedAnimation(animation) => animation.inner.drive(now),
                _ => unreachable!(),
            }
        } else {
            false
        }
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

    pub(crate) fn mark_node_as_clean(&mut self, node_id: NodeId) {
        self.nodes[node_id].state = NodeState::Clean;
    }

    pub fn publish_event(&mut self, _source_id: NodeId, _event: Rc<dyn Any>) {}

    pub fn track(&mut self, source_id: NodeId, scope: Scope) {
        if let Scope::Node(node_id) = scope {
            self.subscriptions.add_node_subscription(source_id, node_id);
        }
    }

    pub fn track_parameter(&mut self, source_id: ParameterId, scope: Scope) {
        if let Scope::Node(node_id) = scope {
            self.subscriptions
                .add_parameter_subscription(source_id, node_id);
        }
    }

    pub fn track_widget_status(
        &mut self,
        widget_id: WidgetId,
        status_mask: WidgetStatusFlags,
        scope: Scope,
    ) {
        if let Scope::Node(node_id) = scope {}
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
