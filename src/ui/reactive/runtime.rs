use super::{
    NodeId,
    animation::{AnimationState, DerivedAnimationState},
    cached::CachedState,
    effect::{BindingState, EffectState},
    event_channel::EventHandlerState,
    var::SignalState,
    widget_status::WidgetStatusFlags,
};
use crate::{
    core::{FxHashMap, FxHashSet, FxIndexSet},
    param::{AnyParameterMap, ParamRef, ParameterId},
    ui::{
        WidgetId, Widgets,
        reactive::{LocalContext, LocalContextMut},
        task_queue::{Task, TaskQueue},
    },
};
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use std::{
    any::Any,
    collections::VecDeque,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Instant,
};

pub struct Node {
    pub(crate) node_type: NodeType,
    pub(crate) state: NodeState,
}

#[derive(Debug, Clone, Copy)]
pub enum Owner {
    Root,
    /// The reactive node is owned by a widget, and will be removed
    /// when the widget is removed.
    Widget(WidgetId),
    /// The reactive node is owned by another node (such as a Var being created within a memo)
    /// The node is removed when its parent is removed.
    Node(NodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SourceId {
    Parameter(ParameterId),
    Node(NodeId),
    Widget(WidgetId),
}

#[derive(Debug, Clone, Copy)]
pub enum ReadScope {
    Untracked,
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
    // Sources
    Trigger,
    Signal(SignalState),
    Animation(AnimationState),
    EventEmitter,
    // Derived nodes
    Memo(CachedState),
    DerivedAnimation(DerivedAnimationState),
    // Effects (reactions)
    Effect(EffectState),
    Binding(BindingState),
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

pub(super) struct LeasedNode(NodeId, NodeType);
impl Deref for LeasedNode {
    type Target = NodeType;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
impl DerefMut for LeasedNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

pub struct ReactiveGraph {
    pub(super) nodes: SlotMap<NodeId, Node>,
    pub(crate) parameters: Rc<dyn AnyParameterMap>,
    nodes_owned_by_node: SecondaryMap<NodeId, FxHashSet<NodeId>>,
    nodes_owned_by_widget: SecondaryMap<WidgetId, FxHashSet<NodeId>>,
    pub(super) sources: SecondaryMap<NodeId, SmallVec<[SourceId; 4]>>,
    pub(super) node_observers: SecondaryMap<NodeId, FxIndexSet<NodeId>>,
    pub(super) parameter_observers: FxHashMap<ParameterId, FxIndexSet<NodeId>>,
    pub(super) widget_observers: SecondaryMap<WidgetId, SmallVec<[(NodeId, WidgetStatusFlags); 4]>>,
    pub(crate) node_id_buffer: VecDeque<NodeId>,
    pub(super) pending_animations: FxIndexSet<NodeId>,
}

impl ReactiveGraph {
    pub fn new(parameter_map: Rc<dyn AnyParameterMap>) -> Self {
        let mut parameter_subscriptions = FxHashMap::default();
        for parameter_id in parameter_map.parameter_ids() {
            parameter_subscriptions.insert(parameter_id, FxIndexSet::default());
        }

        Self {
            nodes: Default::default(),
            sources: Default::default(),
            node_observers: Default::default(),
            parameter_observers: parameter_subscriptions,
            widget_observers: Default::default(),
            parameters: parameter_map,
            nodes_owned_by_node: Default::default(),
            nodes_owned_by_widget: Default::default(),
            node_id_buffer: Default::default(),
            pending_animations: Default::default(),
        }
    }

    pub(crate) fn create_node(
        &mut self,
        node_type: NodeType,
        state: NodeState,
        owner: Owner,
    ) -> NodeId {
        let node = Node { node_type, state };
        let id = self.nodes.insert(node);
        self.node_observers.insert(id, FxIndexSet::default());

        match owner {
            Owner::Widget(widget_id) => {
                self.nodes_owned_by_widget
                    .entry(widget_id)
                    .unwrap()
                    .or_default()
                    .insert(id);
            }
            Owner::Node(node_id) => {
                self.nodes_owned_by_node
                    .entry(node_id)
                    .unwrap()
                    .or_default()
                    .insert(id);
            }
            Owner::Root => {}
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
        // Remove the node's subscriptions to other nodes
        let observers = self
            .node_observers
            .remove(id)
            .expect("Missing observers for node");
        for observer_id in observers {
            self.sources[observer_id].retain(|source_id| *source_id != SourceId::Node(id));
        }

        // Remove other nodes' subscriptions to this node
        if let Some(sources) = self.sources.remove(id) {
            for source_id in sources {
                Self::remove_source_from_observer(
                    &mut self.node_observers,
                    &mut self.parameter_observers,
                    &mut self.widget_observers,
                    source_id,
                    id,
                );
            }
        }

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

    pub fn drive_animations(&mut self, widgets: &mut Widgets, task_queue: &mut TaskQueue) {
        let node_ids = std::mem::take(&mut self.pending_animations);
        let now = Instant::now();
        for node_id in node_ids {
            let did_change = self.try_drive_animation(node_id, now);
            if did_change {
                self.notify(widgets, task_queue, node_id);
                // Re-queue the animation for the next frame
                self.pending_animations.insert(node_id);
            }
        }
    }

    fn try_drive_animation(&mut self, node_id: NodeId, now: Instant) -> bool {
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

    pub(super) fn request_animation(&mut self, node_id: NodeId) {
        self.pending_animations.insert(node_id);
    }

    pub fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef<'_> {
        self.parameters
            .get_by_id(parameter_id)
            .expect("Parameter id should be a parameter that exists")
    }

    /// Temporarily remove a node and return it
    pub(super) fn lease_node(&mut self, node_id: NodeId) -> Option<LeasedNode> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            Some(LeasedNode(
                node_id,
                std::mem::replace(&mut node.node_type, NodeType::TmpRemoved),
            ))
        } else {
            None
        }
    }

    /// Return a node that has previously been leased
    pub(super) fn unlease_node(&mut self, mut node: LeasedNode) {
        std::mem::swap(&mut self.nodes[node.0].node_type, &mut node.1);
    }

    pub(crate) fn mark_node_as_clean(&mut self, node_id: NodeId) {
        self.nodes[node_id].state = NodeState::Clean;
    }

    pub(crate) fn notify(
        &mut self,
        widgets: &mut Widgets,
        task_queue: &mut TaskQueue,
        node_id: NodeId,
    ) {
        let mut observers = std::mem::take(&mut self.node_id_buffer);
        observers.clear();
        observers.extend(self.node_observers[node_id].iter());
        self.notify_source_changed(widgets, task_queue, observers);
    }

    fn notify_source_changed(
        &mut self,
        widgets: &mut Widgets,
        task_queue: &mut TaskQueue,
        mut nodes_to_notify: VecDeque<NodeId>,
    ) {
        let mut effects_to_check = FxHashSet::default();

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
                let node = self.get_node_mut(node_id);
                if node.state < new_state {
                    node.state = new_state;
                    match &node.node_type {
                        NodeType::Effect(_)
                        | NodeType::Binding(_)
                        | NodeType::DerivedAnimation(_) => {
                            effects_to_check.insert(node_id);
                        }
                        _ => {}
                    }
                    nodes_to_notify.extend(self.node_observers[node_id].iter());
                }
                i += 1;
            }
        }

        // Swap back the scratch buffer. Saves us from having to reallocate
        std::mem::swap(&mut self.node_id_buffer, &mut nodes_to_notify);

        for node_id in effects_to_check {
            if self.update_sources_if_necessary(widgets, node_id) == NodeState::Dirty {
                let mut node_type = self.lease_node(node_id).unwrap();
                match node_type.deref_mut() {
                    NodeType::Effect(EffectState { f }) => {
                        // Clear the sources, they will be re-populated while running the effect function
                        self.clear_node_sources(node_id);
                        let task = Task::RunEffect {
                            id: node_id,
                            f: Rc::downgrade(f),
                        };
                        task_queue.push(task);
                    }
                    NodeType::Binding(BindingState { f }) => {
                        let task = Task::UpdateBinding {
                            f: Rc::downgrade(f),
                            node_id,
                        };
                        task_queue.push(task);
                    }
                    NodeType::DerivedAnimation(anim) => {
                        // Clear the sources, they will be re-populated while running the reset function
                        self.clear_node_sources(node_id);
                        if anim.reset(node_id, &mut LocalContext::new(widgets, self)) {
                            self.request_animation(node_id);
                        }
                    }
                    _ => unreachable!(),
                }
                self.unlease_node(node_type);
            }
        }
    }

    /// Given a node (effect, binding, derived animation, or cached value), check if any of its sources have changed.
    /// Any invalid cached value will be re-evaluated.
    fn update_sources_if_necessary(&mut self, widgets: &Widgets, node_id: NodeId) -> NodeState {
        let state = self.get_node(node_id).state;
        if state == NodeState::Clean {
            return NodeState::Clean;
        }

        if state == NodeState::Check {
            for source_id in self.sources[node_id].clone() {
                if let SourceId::Node(source_id) = source_id {
                    self.update_cached_value_if_necessary(widgets, source_id);
                    if self.get_node(node_id).state == NodeState::Dirty {
                        return NodeState::Dirty;
                    }
                }
            }
        }

        self.get_node(node_id).state
    }

    pub(super) fn update_cached_value_if_necessary(&mut self, widgets: &Widgets, node_id: NodeId) {
        if self.update_sources_if_necessary(widgets, node_id) == NodeState::Dirty {
            let mut node_type = self.lease_node(node_id).unwrap();
            match node_type.deref_mut() {
                NodeType::Memo(memo) => {
                    // Clear the sources, they will be re-populated while running the memo function
                    self.clear_node_sources(node_id);
                    if memo.eval(node_id, LocalContext::new(widgets, self)) {
                        // Memo eval returned false, meaning that it has changed.
                        for &observer_id in self.node_observers[node_id].iter() {
                            self.nodes[observer_id].state = NodeState::Dirty;
                        }
                    }
                }
                _ => unreachable!(),
            }
            self.unlease_node(node_type);
        }
        self.get_node_mut(node_id).state = NodeState::Clean;
    }

    pub(crate) fn notify_parameter_subscribers(
        &mut self,
        widgets: &mut Widgets,
        task_queue: &mut TaskQueue,
        source_id: ParameterId,
    ) {
        let mut nodes_to_notify = std::mem::take(&mut self.node_id_buffer);
        nodes_to_notify.clear();
        nodes_to_notify.extend(self.parameter_observers.get_mut(&source_id).unwrap().iter());
        self.notify_source_changed(widgets, task_queue, nodes_to_notify);
    }

    pub(crate) fn notify_widget_status_changed(
        &mut self,
        widgets: &mut Widgets,
        task_queue: &mut TaskQueue,
        widget_id: WidgetId,
        change_mask: WidgetStatusFlags,
    ) {
        if let Some(widget_observers) = self.widget_observers.get(widget_id) {
            let mut nodes_to_notify = std::mem::take(&mut self.node_id_buffer);
            nodes_to_notify.clear();
            nodes_to_notify.extend(
                widget_observers
                    .iter()
                    .filter_map(|(node_id, mask)| mask.contains(change_mask).then_some(node_id)),
            );
            self.notify_source_changed(widgets, task_queue, nodes_to_notify);
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
                    &mut self.node_observers,
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
        self.node_observers[source_id].insert(observer_id);
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
