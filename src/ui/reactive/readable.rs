use std::{any::Any, hash::Hash, marker::PhantomData, rc::Rc};

use rustc_hash::FxBuildHasher;

use crate::{
    core::{FxIndexSet, diff},
    param::ParameterId,
    ui::{
        AnyView, AppState, BuildContext, View, ViewSequence, Widget, WidgetId, WidgetMut,
        WidgetRef, WindowId,
        app_state::Task,
        reactive::{
            animation::{AnimationState, DerivedAnimationState},
            effect::BindingState,
        },
    },
};

use super::{
    Accessor, Computed, Effect, NodeId, NodeState, NodeType, Owner, Scope, WatchContext,
    widget_status::WidgetStatusFlags,
};

pub trait ReactiveContext {
    /// Get immutable access to the underlying reactive runtime. This method
    /// is mostly used internally, users should rarely have to interact directly
    /// with the runtime.
    fn app_state(&self) -> &AppState;

    /// Get mutable access to the underlying reactive runtime. This method
    /// is mostly used internally, users should rarely have to interact directly
    /// with the runtime.
    fn app_state_mut(&mut self) -> &mut AppState;

    fn as_create_context(&mut self, owner: Owner) -> LocalCreateContext {
        LocalCreateContext {
            app_state: self.app_state_mut(),
            owner,
        }
    }
}

/// Contexts implementing `CreateContext` allows reactive elements to be created.
pub trait CreateContext: ReactiveContext {
    /// Returns the owner that should be assigned to newly created reactive nodes.
    ///
    /// We use this mechanism to scope nodes to either:
    /// - A widget (`Some(Owner::Widget)`): When the widget is removed, the node is removed
    /// - Another node (`Some(Owner::Node)`): When the other node is removed, the node is removed
    /// - No owner (None): will not be cleaned up until the plugin instance is exited.
    fn owner(&self) -> Option<Owner>;
}

pub(crate) fn create_var_node(
    cx: &mut dyn CreateContext,
    state: super::var::SignalState,
) -> NodeId {
    let owner = cx.owner();
    cx.app_state_mut()
        .runtime
        .create_node(NodeType::Signal(state), NodeState::Clean, owner)
}

pub(crate) fn create_memo_node(
    cx: &mut dyn CreateContext,
    state: super::cached::CachedState,
) -> NodeId {
    let owner = cx.owner();
    cx.app_state_mut()
        .runtime
        .create_node(NodeType::Memo(state), NodeState::Dirty, owner)
}

pub(crate) fn create_effect_node(
    cx: &mut dyn CreateContext,
    state: super::EffectState,
    run_effect: bool,
) -> NodeId {
    let f = Rc::downgrade(&state.f);
    let owner = cx.owner();
    let id =
        cx.app_state_mut()
            .runtime
            .create_node(NodeType::Effect(state), NodeState::Dirty, owner);
    if run_effect {
        cx.app_state_mut().push_task(Task::RunEffect { id, f });
    }
    id
}

pub(crate) fn create_trigger(cx: &mut dyn CreateContext) -> NodeId {
    let owner = cx.owner();
    cx.app_state_mut()
        .runtime
        .create_node(NodeType::Trigger, NodeState::Clean, owner)
}

pub(crate) fn create_node_binding_node(
    cx: &mut dyn CreateContext,
    source: NodeId,
    state: BindingState,
) -> NodeId {
    let owner = cx.owner();
    let graph = &mut cx.app_state_mut().runtime;
    let id = graph.create_node(NodeType::Binding(state), NodeState::Clean, owner);
    graph.add_node_subscription(source, id);
    id
}

pub(crate) fn create_parameter_binding_node(
    cx: &mut dyn CreateContext,
    source: ParameterId,
    state: BindingState,
) -> NodeId {
    let owner = cx.owner();
    let graph = &mut cx.app_state_mut().runtime;
    let id = graph.create_node(NodeType::Binding(state), NodeState::Clean, owner);
    graph.add_parameter_subscription(source, id);
    id
}

pub(crate) fn create_animation_node(cx: &mut dyn CreateContext, state: AnimationState) -> NodeId {
    let owner = cx.owner();
    cx.app_state_mut()
        .runtime
        .create_node(NodeType::Animation(state), NodeState::Clean, owner)
}

pub(crate) fn create_derived_animation_node(
    cx: &mut dyn CreateContext,
    state_fn: impl FnOnce(&mut dyn CreateContext, NodeId) -> DerivedAnimationState,
) -> NodeId {
    let owner = cx.owner();
    let id = cx
        .app_state_mut()
        .runtime
        .create_node(NodeType::TmpRemoved, NodeState::Clean, owner);
    let state = state_fn(cx, id);
    let _ = std::mem::replace(
        &mut cx.app_state_mut().runtime.nodes[id].node_type,
        NodeType::DerivedAnimation(state),
    );
    id
}

pub(crate) fn create_event_emitter(cx: &mut dyn CreateContext) -> NodeId {
    let owner = cx.owner();
    cx.app_state_mut()
        .runtime
        .create_node(NodeType::EventEmitter, NodeState::Clean, owner)
}

/// Allows access to the underlying window (needed for example to create timers and animations)
pub trait ViewContext: CreateContext {
    fn window_id(&self) -> WindowId;
}

/// Allows access to widgets
pub trait WidgetContext {
    fn widget_ref_dyn(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget>;
    fn widget_mut_dyn(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget>;
    fn replace_widget_dun(&mut self, id: WidgetId, view: AnyView);
}

/// Allows to read and subscribe to reactive nodes
pub trait ReadContext: ReactiveContext {
    fn scope(&self) -> Scope;
}

impl dyn ReadContext + '_ {
    pub fn track(&mut self, source_id: NodeId) {
        if let Scope::Node(node_id) = self.scope() {
            self.app_state_mut()
                .runtime
                .add_node_subscription(source_id, node_id);
        }
    }

    pub fn track_parameter(&mut self, source_id: ParameterId) {
        if let Scope::Node(node_id) = self.scope() {
            self.app_state_mut()
                .runtime
                .add_parameter_subscription(source_id, node_id);
        }
    }

    pub fn track_widget_status(&mut self, widget_id: WidgetId, status_mask: WidgetStatusFlags) {
        if let Scope::Node(node_id) = self.scope() {
            self.app_state_mut().runtime.add_widget_status_subscription(
                widget_id,
                status_mask,
                node_id,
            );
        }
    }
}

/// Allows writing to reactive nodes
pub trait WriteContext: ReactiveContext {}

impl dyn WriteContext + '_ {
    pub fn publish_event(&mut self, _source_id: NodeId, event: Rc<dyn Any>) {
        //self.app_state_mut()
        //    .push_task(Task::HandleEvent { f: , event: () });
    }
}

pub struct LocalCreateContext<'a> {
    app_state: &'a mut AppState,
    owner: Owner,
}

impl ReactiveContext for LocalCreateContext<'_> {
    fn app_state(&self) -> &AppState {
        &self.app_state
    }

    fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.app_state
    }
}

impl CreateContext for LocalCreateContext<'_> {
    fn owner(&self) -> Option<Owner> {
        Some(self.owner)
    }
}

pub struct LocalReadContext<'a> {
    app_state: &'a mut AppState,
    scope: Scope,
}

impl<'a> LocalReadContext<'a> {
    pub fn new(app_state: &'a mut AppState, scope: Scope) -> Self {
        Self { app_state, scope }
    }
}

impl ReactiveContext for LocalReadContext<'_> {
    fn app_state(&self) -> &AppState {
        &self.app_state
    }

    fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
    }
}

impl ReadContext for LocalReadContext<'_> {
    fn scope(&self) -> Scope {
        self.scope
    }
}

pub trait ReactiveValue: Into<Accessor<Self::Value>> {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let ret = self.with_ref_untracked(cx, f);
        self.track(cx);
        ret
    }

    fn track(&self, cx: &mut dyn ReadContext);

    /// Get the current value and subscribe to changes
    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref(cx, Self::Value::clone)
    }

    /// Map the current value using `f`
    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R;

    /// Get the current value
    fn get_untracked(&self, cx: &mut dyn ReactiveContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref_untracked(cx, Self::Value::clone)
    }

    fn map<R, F>(self, f: F) -> impl ReactiveValue<Value = R>
    where
        F: Fn(&Self::Value) -> R + 'static,
        R: 'static,
        Self: Sized + 'static,
        Self::Value: 'static,
    {
        Mapped {
            parent: self,
            map_fn: f,
            _marker: PhantomData,
        }
    }

    /// Subscribe to changes to this readable. Whenever the value is updated,
    /// `f` is called.`
    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn WatchContext, &Self::Value) + 'static;

    fn map_to_views_keyed<T, K, V, FKey, FView>(
        self,
        key_fn: FKey,
        view_fn: FView,
    ) -> impl ViewSequence
    where
        Self: 'static,
        for<'a> &'a Self::Value: IntoIterator<Item = &'a T>,
        K: Hash + Eq + 'static,
        T: 'static,
        V: View,
        FView: Fn(&T) -> V + 'static,
        FKey: Fn(&T) -> K + 'static,
    {
        MapToViewsKeyedImpl {
            readable: self,
            view_fn,
            key_fn,
        }
    }
}
struct MapToViewsKeyedImpl<R, F, FKey> {
    readable: R,
    view_fn: F,
    key_fn: FKey,
}

impl<S, C: 'static, K, T, V, F, FKey> ViewSequence for MapToViewsKeyedImpl<S, F, FKey>
where
    S: ReactiveValue<Value = C> + 'static,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    K: Hash + Eq + 'static,
    T: 'static,
    V: View,
    F: Fn(&T) -> V + 'static,
    FKey: Fn(&T) -> K + 'static,
{
    fn build_seq(self, cx: &mut BuildContext<dyn Widget>) {
        let views_keys: Vec<_> = self.readable.with_ref(cx, |values| {
            values
                .into_iter()
                .map(|value| ((self.key_fn)(value), (self.view_fn)(value)))
                .collect()
        });

        let mut old_indices = FxIndexSet::with_capacity_and_hasher(views_keys.len(), FxBuildHasher);
        for (key, view) in views_keys.into_iter() {
            old_indices.insert(key);
            cx.add_child(view);
        }

        let widget_id = cx.id();
        self.readable.watch(cx, move |cx, values| {
            let new_indices: FxIndexSet<_> = values.into_iter().map(|x| (self.key_fn)(x)).collect();
            let value_vec: Vec<_> = values.into_iter().collect();
            let mut widget = cx.widget_mut(widget_id);

            diff::diff_keyed_with(&old_indices, &new_indices, &value_vec, |diff| {
                let f = |x: &&T| (self.view_fn)(*x);
                widget.apply_diff_to_children(diff, &f)
            });
            widget.request_render();

            old_indices = new_indices;
        });
    }
}

#[derive(Clone, Copy)]
pub struct Mapped<S, T, R, F> {
    parent: S,
    map_fn: F,
    _marker: PhantomData<fn(&T) -> R>,
}

impl<S, T, R, F> From<Mapped<S, T, R, F>> for Accessor<R>
where
    T: 'static,
    R: 'static,
    S: ReactiveValue<Value = T> + 'static,
    F: Fn(&T) -> R + 'static,
{
    fn from(value: Mapped<S, T, R, F>) -> Self {
        Self::Computed(Computed::new(move |cx| {
            value.parent.with_ref(cx, |x| (value.map_fn)(x))
        }))
    }
}

impl<S, T, R, F> ReactiveValue for Mapped<S, T, R, F>
where
    S: ReactiveValue<Value = T> + 'static,
    T: Any,
    R: 'static,
    F: Fn(&T) -> R + 'static,
{
    type Value = R;

    fn track(&self, cx: &mut dyn ReadContext) {
        self.parent.track(cx);
    }

    fn with_ref<R2>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R2) -> R2 {
        self.parent.with_ref(cx, |x| f(&(self.map_fn)(x)))
    }

    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value {
        self.parent.with_ref(cx, |x| (self.map_fn)(x))
    }

    fn with_ref_untracked<R2>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R2,
    ) -> R2 {
        self.parent.with_ref_untracked(cx, |x| f(&(self.map_fn)(x)))
    }

    fn get_untracked(&self, cx: &mut dyn ReactiveContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.parent.with_ref_untracked(cx, |x| (self.map_fn)(x))
    }

    fn watch<F2>(self, cx: &mut dyn CreateContext, mut f: F2) -> Effect
    where
        F2: FnMut(&mut dyn WatchContext, &Self::Value) + 'static,
    {
        self.parent.watch(cx, move |cx, value| {
            let mapped_value = (self.map_fn)(value);
            f(cx, &mapped_value);
        })
    }
}
