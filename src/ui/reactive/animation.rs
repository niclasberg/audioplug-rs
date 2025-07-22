use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    time::Instant,
};

use crate::{
    core::{Interpolate, SpringPhysics},
    ui::{AppState, WindowId, reactive::notify},
};

use super::{
    Accessor, CreateContext, Effect, LocalReadContext, NodeId, NodeType, ReactiveContext,
    ReactiveValue, ReadContext, ReadSignal, Scope, ViewContext, WatchContext, WriteContext,
};

pub use super::spring::{SpringAnimation, SpringOptions};
pub use super::tween::{Easing, TweenAnimation, TweenOptions};

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
pub(super) trait AnyAnimation {
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
    pub(super) inner: Box<dyn AnyAnimation>,
    pub(super) window_id: WindowId,
}

type ResetFn = dyn Fn(&mut dyn AnyAnimation, &mut dyn ReadContext) -> bool;

pub struct DerivedAnimationState {
    pub(super) inner: Box<dyn AnyAnimation>,
    reset_fn: Box<ResetFn>,
    pub(super) window_id: WindowId,
}

impl DerivedAnimationState {
    pub fn reset(&mut self, node_id: NodeId, app_state: &mut AppState) -> bool {
        (self.reset_fn)(
            &mut self.inner,
            &mut LocalReadContext::new(app_state, Scope::Node(node_id)),
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
        let inner = Box::new(animation);
        let state = AnimationState { inner, window_id };
        Self {
            id: cx.create_animation_node(state),
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
        let node = cx.app_state_mut().runtime.get_node_mut(self.id);
        let window_id_if_changed = match &mut node.node_type {
            NodeType::Animation(anim) => {
                anim.inner.set_target_dyn(&value).then_some(anim.window_id)
            }
            _ => unreachable!(),
        };
        if let Some(window_id) = window_id_if_changed {
            cx.app_state_mut().request_animation(window_id, self.id);
        }
    }

    pub fn set_target_and_value(&self, cx: &mut dyn WriteContext, value: T) {
        let node = cx.app_state_mut().runtime.get_node_mut(self.id);
        match &mut node.node_type {
            NodeType::Animation(anim) => anim.inner.set_target_and_value_dyn(&value),
            _ => unreachable!(),
        };
        notify(cx.app_state_mut(), self.id);
    }
}

impl<T: 'static> From<Animated<T>> for ReadSignal<T> {
    fn from(value: Animated<T>) -> Self {
        ReadSignal::from_node(value.id)
    }
}

impl<T: 'static> From<Animated<T>> for Accessor<T> {
    fn from(value: Animated<T>) -> Self {
        Self::ReadSignal(ReadSignal::from_node(value.id))
    }
}

impl<T: 'static> ReactiveValue for Animated<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        cx.track(self.id);
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        cx.track(self.id);
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

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node(cx, self.id, f)
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

impl<T: 'static> From<AnimatedFn<T>> for ReadSignal<T> {
    fn from(value: AnimatedFn<T>) -> Self {
        ReadSignal::from_node(value.id)
    }
}

impl<T> From<AnimatedFn<T>> for Accessor<T> {
    fn from(value: AnimatedFn<T>) -> Self {
        Self::ReadSignal(ReadSignal::from_node(value.id))
    }
}

impl<T: 'static> ReactiveValue for AnimatedFn<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        cx.track(self.id);
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        cx.track(self.id);
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

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node::<T>(cx, self.id, f)
    }
}

fn get_animation_value_ref(cx: &dyn ReactiveContext, node_id: NodeId) -> &dyn Any {
    match &cx.app_state().runtime.get_node(node_id).node_type {
        NodeType::Animation(animation) => animation.inner.value_dyn(),
        NodeType::DerivedAnimation(animation) => animation.inner.value_dyn(),
        _ => unreachable!(),
    }
}
