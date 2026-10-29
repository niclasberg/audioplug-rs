use slotmap::{Key, SecondaryMap, SlotMap};
use smallvec::SmallVec;

use crate::{
    core::FxHashMap,
    param::ParameterId,
    ui::{
        WidgetId,
        reactive::{
            Node, NodeId, NodeState, NodeType, Owner, Source, WidgetPhase, WidgetStatusFlags,
        },
    },
};

type SourceMap = SecondaryMap<NodeId, SmallVec<[Source; 4]>>;
type NodeObservers = SecondaryMap<NodeId, SmallVec<[NodeId; 4]>>;
type ParameterObservers = FxHashMap<ParameterId, SmallVec<[NodeId; 4]>>;

pub struct ReactiveGraph {
    /// State of all reactive nodes in the graph, e.g. Cached, Var, Animations etc.
    pub(super) nodes: SlotMap<NodeId, Node>,
    /// Node sources, the upstream nodes that each node depend on
    pub(super) sources: SourceMap,
    pub(super) node_observers: NodeObservers,
    pub(super) parameter_observers: ParameterObservers,
    pub(super) reactive_widget_states: SecondaryMap<WidgetId, ReactiveWidgetState>,
}

#[derive(Default, Clone)]
pub struct ReactiveWidgetState {
    pub(super) observers: SmallVec<[(NodeId, WidgetStatusFlags); 4]>,
    pub(super) first_owned_node_id: NodeId,
    pub(super) sync_widget_node_id: NodeId,
    pub(super) request_render_node_id: NodeId,
}

impl ReactiveGraph {
    pub fn new(parameter_observers: ParameterObservers) -> Self {
        Self {
            nodes: Default::default(),
            sources: Default::default(),
            node_observers: Default::default(),
            parameter_observers,
            reactive_widget_states: Default::default(),
        }
    }

    pub(crate) fn create_node(
        &mut self,
        node_type: NodeType,
        state: NodeState,
        owner: Owner,
    ) -> NodeId {
        let id = self.nodes.insert(Node {
            node_type,
            state,
            owner,
            first_child_id: NodeId::null(),
            next_sibling_id: NodeId::null(),
        });

        let next_sibling_id = match owner {
            Owner::Widget(widget_id) => std::mem::replace(
                &mut self
                    .reactive_widget_states
                    .entry(widget_id)
                    .unwrap()
                    .or_default()
                    .first_owned_node_id,
                id,
            ),
            Owner::Node(parent_id) => {
                std::mem::replace(&mut self.nodes[parent_id].first_child_id, id)
            }
            Owner::Root => NodeId::null(),
        };
        self.nodes[id].next_sibling_id = next_sibling_id;

        id
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Node {
        // Remove the node's subscriptions to other nodes
        if let Some(observers) = self.node_observers.remove(node_id) {
            for observer_id in observers {
                self.sources[observer_id].retain(|source_id| *source_id != Source::Node(node_id));
            }
        }

        // Remove other nodes' subscriptions to this node
        if let Some(sources) = self.sources.remove(node_id) {
            for source_id in sources {
                self.remove_source_from_observer(source_id, node_id);
            }
        }

        let node = self.nodes.remove(node_id).expect("Missing node");
        self.remove_node_and_siblings(node.first_child_id);
        node
    }

    fn remove_node_and_siblings(&mut self, mut id: NodeId) {
        while !id.is_null() {
            let node = self.remove_node(id);
            id = node.next_sibling_id;
        }
    }

    /// Removes the widget and all owned reactive nodes from the graph
    pub fn clear_widget(&mut self, widget_id: WidgetId) {
        if let Some(reactive_widget_state) = self.reactive_widget_states.remove(widget_id) {
            self.remove_node_and_siblings(reactive_widget_state.first_owned_node_id);
        }
    }

    pub fn clear_node_sources(&mut self, node_id: NodeId) {
        if let Some(sources) = self.sources.get_mut(node_id) {
            let sources = std::mem::take(sources);
            for source_id in sources {
                self.remove_source_from_observer(source_id, node_id);
            }
        }
        let first_child_id =
            std::mem::replace(&mut self.nodes[node_id].first_child_id, NodeId::null());
        self.remove_node_and_siblings(first_child_id);
    }

    fn remove_source_from_observer(&mut self, source_id: Source, observer_id: NodeId) {
        match source_id {
            Source::Parameter(parameter_id) => {
                self.parameter_observers
                    .get_mut(&parameter_id)
                    .expect("Missing parameter subscription")
                    .retain(|id| *id != observer_id);
            }
            Source::Node(node_id) => {
                self.node_observers[node_id].retain(|id| *id != observer_id);
            }
            Source::Widget(widget_id, status_mask) => {
                self.reactive_widget_states[widget_id]
                    .observers
                    .retain(|w| *w != (observer_id, status_mask));
            }
        }
    }

    pub fn add_parameter_subscription(&mut self, source_id: ParameterId, observer_id: NodeId) {
        let observers = self.parameter_observers.get_mut(&source_id).unwrap();
        if !observers.contains(&observer_id) {
            observers.push(observer_id);
            self.sources[observer_id].push(Source::Parameter(source_id));
        }
    }

    pub fn add_node_subscription(&mut self, source_id: NodeId, observer_id: NodeId) {
        let observers = &mut self.node_observers.entry(source_id).unwrap().or_default();
        if !observers.contains(&observer_id) {
            self.node_observers[source_id].push(observer_id);
            self.sources
                .entry(observer_id)
                .unwrap()
                .or_default()
                .push(Source::Node(source_id));
        }
    }

    pub fn add_widget_status_subscription(
        &mut self,
        widget_id: WidgetId,
        status_mask: WidgetStatusFlags,
        observer_id: NodeId,
    ) {
        // Maybe check if it exists and then merge masks?
        self.reactive_widget_states
            .entry(widget_id)
            .unwrap()
            .or_default()
            .observers
            .push((observer_id, status_mask));
        self.sources
            .entry(observer_id)
            .unwrap()
            .or_default()
            .push(Source::Widget(widget_id, status_mask));
    }

    pub fn node_observers_iter(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> {
        self.node_observers
            .get(node_id)
            .into_iter()
            .flatten()
            .copied()
    }

    pub fn parameter_observers_iter(&self, param_id: ParameterId) -> impl Iterator<Item = NodeId> {
        self.parameter_observers
            .get(&param_id)
            .into_iter()
            .flatten()
            .copied()
    }

    pub fn widget_status_observers_iter(
        &self,
        widget_id: WidgetId,
        change_mask: WidgetStatusFlags,
    ) -> impl Iterator<Item = NodeId> {
        self.reactive_widget_states
            .get(widget_id)
            .map(|reactive_widget_state| &reactive_widget_state.observers)
            .into_iter()
            .flatten()
            .filter_map(move |(node_id, mask)| mask.contains(change_mask).then_some(*node_id))
    }

    pub fn widget_invalidation_node_id(
        &mut self,
        widget_id: WidgetId,
        phase: WidgetPhase,
    ) -> NodeId {
        let reactive_widget_state = self
            .reactive_widget_states
            .entry(widget_id)
            .unwrap()
            .or_default();
        let existing_node_id = match phase {
            WidgetPhase::SyncWidget => reactive_widget_state.sync_widget_node_id,
            WidgetPhase::Render => reactive_widget_state.request_render_node_id,
        };
        if !existing_node_id.is_null() {
            return existing_node_id;
        }

        let node_id = self.create_node(
            NodeType::InvalidateWidget { widget_id, phase },
            NodeState::Check,
            Owner::Widget(widget_id),
        );
        match phase {
            WidgetPhase::SyncWidget => {
                self.reactive_widget_states[widget_id].sync_widget_node_id = node_id
            }
            WidgetPhase::Render => {
                self.reactive_widget_states[widget_id].request_render_node_id = node_id
            }
        };

        node_id
    }

    pub fn mark_observers_dirty(&mut self, node_id: NodeId) {
        if let Some(observers) = self.node_observers.get(node_id) {
            for observer_id in observers.iter().copied() {
                self.nodes[observer_id].state = NodeState::Dirty;
            }
        }
    }

    #[inline(always)]
    pub fn get_node_state(&self, node_id: NodeId) -> NodeState {
        self.nodes[node_id].state
    }

    #[inline(always)]
    pub fn set_node_state(&mut self, node_id: NodeId, node_state: NodeState) -> NodeState {
        std::mem::replace(&mut self.nodes[node_id].state, node_state)
    }
}
