use std::{any::Any, rc::Rc};

use crate::{
    param::{ParamRef, ParameterId},
    ui::{
        AnyView, HostHandle, Widget, WidgetId, WidgetMut, WidgetRef, Widgets,
        reactive::runtime::Node,
        task_queue::{Task, TaskQueue},
    },
};

use super::{
    EffectState, NodeId, ReactiveGraph, ReadScope,
    animation::{AnimationState, DerivedAnimationState},
    cached::CachedState,
    effect::WatchState,
    runtime::{NodeState, NodeType, Owner},
    var::SignalState,
    widget_status::WidgetStatusFlags,
};

pub struct ReadContext<'a> {
    pub(crate) widgets: &'a Widgets,
    pub(crate) reactive_graph: &'a mut ReactiveGraph,
    pub(crate) scope: ReadScope,
}

impl ReadContext<'_> {
    pub fn track(&mut self, source_id: NodeId) {
        if let ReadScope::Node(node_id) = self.scope {
            self.reactive_graph
                .add_node_subscription(source_id, node_id);
        }
    }

    pub fn track_parameter(&mut self, source_id: ParameterId) {
        if let ReadScope::Node(node_id) = self.scope {
            self.reactive_graph
                .add_parameter_subscription(source_id, node_id);
        }
    }

    pub fn track_widget_status(&mut self, widget_id: WidgetId, status_mask: WidgetStatusFlags) {
        if let ReadScope::Node(node_id) = self.scope {
            self.reactive_graph
                .add_widget_status_subscription(widget_id, status_mask, node_id);
        }
    }

    pub fn update_value_if_needed(&mut self, node_id: NodeId) {
        self.reactive_graph
            .update_cached_value_if_necessary(self.widgets, node_id);
    }

    pub fn get_node_value_ref(&self, node_id: NodeId) -> Option<&dyn Any> {
        self.reactive_graph
            .try_get_node(node_id)
            .map(|node| NodeType::get_value_ref(&node.node_type))
    }

    pub fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef<'_> {
        self.reactive_graph.get_parameter_ref(parameter_id)
    }

    pub fn with_read_scope(&mut self, scope: ReadScope) -> ReadContext<'_> {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope,
        }
    }
}

/// Allows to read from reactive nodes
pub trait CanRead<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2;
}

impl<'s> CanRead<'s> for ReadContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: self.scope,
        }
    }
}

pub struct CreateContext<'a> {
    pub(crate) widgets: &'a mut Widgets,
    pub(crate) reactive_graph: &'a mut ReactiveGraph,
    pub(crate) task_queue: &'a mut TaskQueue,
    pub(crate) owner: Owner,
}

impl<'a> CreateContext<'a> {
    pub(crate) fn new_root_context(
        widgets: &'a mut Widgets,
        reactive_graph: &'a mut ReactiveGraph,
        task_queue: &'a mut TaskQueue,
    ) -> Self {
        Self {
            widgets,
            reactive_graph,
            task_queue,
            owner: Owner::Root,
        }
    }

    pub(crate) fn create_var_node(&mut self, state: SignalState) -> NodeId {
        self.reactive_graph.create_node(
            NodeType::Signal(state),
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        )
    }

    pub(crate) fn create_memo_node(&mut self, state: CachedState) -> NodeId {
        self.reactive_graph.create_node(
            NodeType::Memo(state),
            NodeState::Dirty,
            self.owner,
            &mut self.widgets.tree,
        )
    }

    pub(crate) fn create_effect_node(&mut self, state: EffectState, run_effect: bool) -> NodeId {
        let f = Rc::downgrade(&state.f);
        let id = self.reactive_graph.create_node(
            NodeType::Effect(state),
            NodeState::Dirty,
            self.owner,
            &mut self.widgets.tree,
        );
        if run_effect {
            self.task_queue.push(Task::RunEffect { id, f });
        }
        id
    }

    pub(crate) fn create_trigger(&mut self) -> NodeId {
        self.reactive_graph.create_node(
            NodeType::Trigger,
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        )
    }

    pub(crate) fn create_node_watcher(&mut self, source: NodeId, state: WatchState) -> NodeId {
        let graph = &mut self.reactive_graph;
        let id = graph.create_node(
            NodeType::Binding(state),
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        );
        graph.add_node_subscription(source, id);
        id
    }

    pub(crate) fn create_parameter_watcher(
        &mut self,
        source: ParameterId,
        state: WatchState,
    ) -> NodeId {
        let graph = &mut self.reactive_graph;
        let id = graph.create_node(
            NodeType::Binding(state),
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        );
        graph.add_parameter_subscription(source, id);
        id
    }

    pub(crate) fn create_widget_status_watcher(
        &mut self,
        widget: WidgetId,
        status_mask: WidgetStatusFlags,
        state: WatchState,
    ) -> NodeId {
        let graph = &mut self.reactive_graph;
        let id = graph.create_node(
            NodeType::Binding(state),
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        );
        graph.add_widget_status_subscription(widget, status_mask, id);
        id
    }

    pub(crate) fn create_animation_node(&mut self, state: AnimationState) -> NodeId {
        self.reactive_graph.create_node(
            NodeType::Animation(state),
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        )
    }

    pub(crate) fn create_derived_animation_node(
        &mut self,
        state_fn: impl FnOnce(&mut CreateContext, NodeId) -> DerivedAnimationState,
    ) -> NodeId {
        let id = self.reactive_graph.create_node(
            NodeType::TmpRemoved,
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        );
        let state = state_fn(self, id);
        let _ = std::mem::replace(
            &mut self.reactive_graph.nodes[id].node_type,
            NodeType::DerivedAnimation(state),
        );
        id
    }

    pub(crate) fn create_event_emitter(&mut self) -> NodeId {
        self.reactive_graph.create_node(
            NodeType::EventEmitter,
            NodeState::Clean,
            self.owner,
            &mut self.widgets.tree,
        )
    }
}

/// Contexts implementing `CanCreate` allows reactive elements to be created.
pub trait CanCreate<'s>: CanRead<'s> {
    fn create_context<'s2>(&'s2 mut self) -> CreateContext<'s2>
    where
        's: 's2;
}

impl<'s> CanCreate<'s> for CreateContext<'s> {
    fn create_context<'s2>(&'s2 mut self) -> CreateContext<'s2>
    where
        's: 's2,
    {
        CreateContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
            owner: self.owner,
        }
    }
}
// Allow untracked reads while writing
impl<'s> CanRead<'s> for CreateContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Untracked,
        }
    }
}

/// Allows access to widgets
pub trait WidgetContext {
    fn widget_ref_dyn(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget>;
    fn widget_mut_dyn(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget>;
    fn replace_widget_dyn(&mut self, id: WidgetId, view: AnyView);
}

pub struct WriteContext<'a> {
    pub(crate) widgets: &'a mut Widgets,
    pub(crate) reactive_graph: &'a mut ReactiveGraph,
    pub(crate) task_queue: &'a mut TaskQueue,
    pub(crate) host_handle: Option<&'a dyn HostHandle>,
}

impl<'a> WriteContext<'a> {
    pub fn get_parameter_ref(&self, parameter_id: ParameterId) -> ParamRef<'_> {
        self.reactive_graph.get_parameter_ref(parameter_id)
    }

    pub fn notify(&mut self, node_id: NodeId) {
        self.reactive_graph
            .notify(&self.widgets, &mut self.task_queue, node_id);
    }

    pub(super) fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node {
        &mut self.reactive_graph.nodes[node_id]
    }

    pub fn request_animation(&mut self, node_id: NodeId) {
        self.reactive_graph.pending_animations.insert(node_id);
    }

    pub fn notify_parameter_subscribers(&mut self, parameter_id: ParameterId) {
        self.reactive_graph.notify_parameter_subscribers(
            self.widgets,
            self.task_queue,
            parameter_id,
        );
    }

    pub fn notify_widget_status_changed(
        &mut self,
        widget_id: WidgetId,
        change_mask: WidgetStatusFlags,
    ) {
        self.reactive_graph.notify_widget_status_changed(
            self.widgets,
            self.task_queue,
            widget_id,
            change_mask,
        );
    }

    pub fn host_handle(&self) -> &dyn HostHandle {
        self.host_handle.unwrap()
    }

    pub fn as_create_context(&mut self, owner: Owner) -> CreateContext<'_> {
        CreateContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
            owner,
        }
    }
}

/// Allows writing to reactive nodes
pub trait CanWrite<'s>: CanRead<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2;
}

impl<'s> CanWrite<'s> for WriteContext<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2,
    {
        WriteContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            task_queue: self.task_queue,
            host_handle: self.host_handle,
        }
    }
}

// Allow untracked reads while writing
impl<'s> CanRead<'s> for WriteContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Untracked,
        }
    }
}
