use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    time::Instant,
};

use crate::{
    core::{Interpolate, SpringPhysics},
    AnimationFrame,
};

use super::{
    accessor::SourceId, layout::request_layout, render::invalidate_widget, AppState,
    LocalReadContext, NodeId, NodeType, ReactiveContext, ReadContext, Readable, Runtime, Scope,
    ViewContext, WidgetId, WindowId, WriteContext,
};

mod spring;
mod tween;

pub use spring::{SpringAnimation, SpringOptions};
pub use tween::{Easing, TweenAnimation, TweenOptions};

/// Should be called when the animation timer for a window ticks.
/// Steps all animations that have been enqueued for window.
pub(super) fn drive_animations(
    app_state: &mut AppState,
    window_id: WindowId,
    animation_frame: AnimationFrame,
) {
    let widget_ids = std::mem::take(&mut app_state.window_mut(window_id).pending_widget_animations);
    for widget_id in widget_ids {
        let mut ctx = AnimationContext {
            id: widget_id,
            app_state,
        };
        ctx.run_animation(animation_frame)
    }

    let node_ids = std::mem::take(&mut app_state.window_mut(window_id).pending_node_animations);
    let now = Instant::now();
    for node_id in node_ids {
        let did_change = if let Some(node) = app_state.runtime_mut().try_get_node_mut(node_id) {
            match &mut node.node_type {
                NodeType::Animation(animation) => animation.inner.drive(now),
                NodeType::DerivedAnimation(animation) => animation.inner.drive(now),
                _ => unreachable!(),
            }
        } else {
            false
        };

        if did_change {
            app_state.runtime_mut().notify(node_id);
            // Re-queue the animation for the next frame
            app_state
                .window_mut(window_id)
                .pending_node_animations
                .insert(node_id);
        }
    }

    app_state.run_effects();
}

pub fn request_animation_frame(app_state: &mut AppState, widget_id: WidgetId) {
    let window_id = app_state.get_window_id_for_widget(widget_id);
    app_state
        .window_mut(window_id)
        .pending_widget_animations
        .insert(widget_id);
}

pub struct AnimationContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl AnimationContext<'_> {
    fn run_animation(&mut self, animation_frame: AnimationFrame) {
        if let Some(mut widget) = self.app_state.widgets.remove(self.id) {
            widget.animation_frame(animation_frame, self);
            self.app_state.widgets.insert(self.id, widget);
        }
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn request_render(&mut self) {
        invalidate_widget(self.app_state, self.id);
    }

    pub fn request_animation(&mut self) {
        request_animation_frame(self.app_state, self.id)
    }

    pub fn request_layout(&mut self) {
        request_layout(self.app_state, self.id);
    }
}

pub trait Animation {
    type Value;

    /// Get the current value
    fn value(&self) -> &Self::Value;
    /// Drives the animation
    /// Should return false when the animation is finished, and true otherwise
    fn drive(&mut self, time: Instant) -> bool;
    fn set_target(&mut self, value: &Self::Value) -> bool;
    fn set_target_and_value(&mut self, value: &Self::Value);
}

/// Type-erased animation, needed so that we can store them in the runtime
trait AnyAnimation {
    fn value_dyn(&self) -> &dyn Any;
    fn drive(&mut self, time: Instant) -> bool;
    fn set_target_dyn(&mut self, value: &dyn Any) -> bool;
    fn set_target_and_value_dyn(&mut self, value: &dyn Any);
}

impl<A: Animation> AnyAnimation for A
where
    A::Value: Any,
{
    fn value_dyn(&self) -> &dyn Any {
        self.value()
    }

    fn drive(&mut self, time: Instant) -> bool {
        self.drive(time)
    }

    fn set_target_dyn(&mut self, value: &dyn Any) -> bool {
        self.set_target(value.downcast_ref().unwrap())
    }

    fn set_target_and_value_dyn(&mut self, value: &dyn Any) {
        self.set_target_and_value(value.downcast_ref().unwrap());
    }
}

impl AnyAnimation for Box<dyn AnyAnimation> {
    fn value_dyn(&self) -> &dyn Any {
        self.deref().value_dyn()
    }

    fn drive(&mut self, time: Instant) -> bool {
        self.deref_mut().drive(time)
    }

    fn set_target_dyn(&mut self, value: &dyn Any) -> bool {
        self.deref_mut().set_target_dyn(value)
    }

    fn set_target_and_value_dyn(&mut self, value: &dyn Any) {
        self.deref_mut().set_target_and_value_dyn(value);
    }
}

pub struct AnimationState {
    inner: Box<dyn AnyAnimation>,
    pub(super) window_id: WindowId,
}

type ResetFn = dyn Fn(&mut dyn AnyAnimation, &mut dyn ReadContext) -> bool;

pub struct DerivedAnimationState {
    inner: Box<dyn AnyAnimation>,
    reset_fn: Box<ResetFn>,
    pub(super) window_id: WindowId,
}

impl DerivedAnimationState {
    pub fn reset(&mut self, node_id: NodeId, runtime: &mut Runtime) -> bool {
        (self.reset_fn)(
            &mut self.inner,
            &mut LocalReadContext::new(runtime, Scope::Node(node_id)),
        )
    }
}

pub struct Animated<T> {
    pub(crate) id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T> Clone for Animated<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Animated<T> {}

impl<T: 'static> Animated<T> {
    pub fn new(cx: &mut dyn ViewContext, animation: impl Animation<Value = T> + 'static) -> Self {
        let window_id = cx.window_id();
        let owner = cx.owner();
        let inner = Box::new(animation);
        let state = AnimationState { inner, window_id };
        Self {
            id: cx.runtime_mut().create_animation_node(state, owner),
            _phantom: PhantomData,
        }
    }

    pub fn spring(cx: &mut dyn ViewContext, initial_value: T, options: SpringOptions) -> Self
    where
        T: SpringPhysics + Clone + Any,
    {
        Self::new(cx, SpringAnimation::new(initial_value, options))
    }

    pub fn tween(cx: &mut dyn ViewContext, initial_value: T, options: TweenOptions) -> Self
    where
        T: Interpolate + Clone + Any,
    {
        Self::new(cx, TweenAnimation::new(initial_value, options))
    }
}

impl<T: Any> Animated<T> {
    pub fn set_target(&self, cx: &mut dyn WriteContext, value: T) {
        let node = cx.runtime_mut().get_node_mut(self.id);
        let window_id_if_changed = match &mut node.node_type {
            NodeType::Animation(anim) => {
                anim.inner.set_target_dyn(&value).then_some(anim.window_id)
            }
            _ => unreachable!(),
        };
        if let Some(window_id) = window_id_if_changed {
            cx.runtime_mut().request_animation(window_id, self.id);
        }
    }

    pub fn set_target_and_value(&self, cx: &mut dyn WriteContext, value: T) {
        let node = cx.runtime_mut().get_node_mut(self.id);
        match &mut node.node_type {
            NodeType::Animation(anim) => anim.inner.set_target_and_value_dyn(&value),
            _ => unreachable!(),
        };
        cx.runtime_mut().notify(self.id);
    }
}

impl<T: 'static> Readable for Animated<T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }
}

pub struct AnimatedFn<T> {
    pub(crate) id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T> Clone for AnimatedFn<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AnimatedFn<T> {}

impl<T: 'static> AnimatedFn<T> {
    pub fn new<A: Animation<Value = T> + 'static>(
        cx: &mut dyn ViewContext,
        f_value: impl Fn(&mut dyn ReadContext) -> T + 'static,
        f_anim: impl FnOnce(T) -> A,
    ) -> Self {
        let window_id = cx.window_id();
        let owner = cx.owner();
        let id = cx.runtime_mut().create_derived_animation_node(
            move |runtime, id| {
                let value = f_value(&mut LocalReadContext::new(runtime, Scope::Node(id)));
                let inner = Box::new(f_anim(value));
                let reset_fn = Box::new(
                    move |animation: &mut dyn AnyAnimation, cx: &mut dyn ReadContext| {
                        animation.set_target_dyn(&f_value(cx))
                    },
                );
                DerivedAnimationState {
                    inner,
                    reset_fn,
                    window_id,
                }
            },
            owner,
        );

        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn spring(
        cx: &mut dyn ViewContext,
        f: impl Fn(&mut dyn ReadContext) -> T + 'static,
        options: SpringOptions,
    ) -> Self
    where
        T: SpringPhysics + Clone + Any,
    {
        Self::new(cx, f, move |initial_value| {
            SpringAnimation::new(initial_value, options)
        })
    }

    pub fn tween(
        cx: &mut dyn ViewContext,
        f: impl Fn(&mut dyn ReadContext) -> T + 'static,
        options: TweenOptions,
    ) -> Self
    where
        T: Interpolate + Clone + Any,
    {
        Self::new(cx, f, move |initial_value| {
            TweenAnimation::new(initial_value, options)
        })
    }
}

impl<T: 'static> Readable for AnimatedFn<T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        let scope = cx.scope();
        cx.runtime_mut().track(self.id, scope);
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("AnimationFn value had wrong type"))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("AnimationFn value had wrong type"))
    }
}

fn get_animation_value_ref(cx: &mut dyn ReactiveContext, node_id: NodeId) -> &dyn Any {
    match &cx.runtime().get_node(node_id).node_type {
        NodeType::Animation(animation) => animation.inner.value_dyn(),
        NodeType::DerivedAnimation(animation) => animation.inner.value_dyn(),
        _ => unreachable!(),
    }
}
