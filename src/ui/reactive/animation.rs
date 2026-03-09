use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    time::Instant,
};

use super::{
    CanCreate, CanRead, CanWrite, Effect, NodeId, ReactiveValue, ReadScope, ReadSignal,
    WatchContext, runtime::NodeType,
};
use crate::ui::{ViewProp, Widgets, reactive::ReactiveGraph};
use crate::{
    core::{Lerp, SpringPhysics},
    ui::reactive::ReadContext,
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
}

type ResetFn = dyn Fn(&mut dyn AnyAnimation, &mut ReactiveGraph, &Widgets) -> bool;

pub struct DerivedAnimationState {
    pub(super) inner: Box<dyn AnyAnimation>,
    reset_fn: Box<ResetFn>,
}

impl DerivedAnimationState {
    pub fn reset(&mut self, reactive_graph: &mut ReactiveGraph, widgets: &Widgets) -> bool {
        (self.reset_fn)(&mut self.inner, reactive_graph, widgets)
    }
}

pub struct AnimatedVar<T> {
    pub(crate) id: NodeId,
    _phantom: PhantomData<*const T>,
}

impl<T> Clone for AnimatedVar<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AnimatedVar<T> {}

impl<T: 'static> AnimatedVar<T> {
    pub fn new<'a>(
        cx: &mut impl CanCreate<'a>,
        animation: impl Animation<Value = T> + 'static,
    ) -> Self {
        let inner = Box::new(animation);
        let state = AnimationState { inner };
        Self {
            id: cx.create_context().create_animation_node(state),
            _phantom: PhantomData,
        }
    }

    pub fn spring<'a>(cx: &mut impl CanCreate<'a>, initial_value: T, options: SpringOptions) -> Self
    where
        T: SpringPhysics + Clone + Any,
    {
        Self::new(cx, SpringAnimation::new(initial_value, options))
    }

    pub fn tween<'a>(cx: &mut impl CanCreate<'a>, initial_value: T, options: TweenOptions) -> Self
    where
        T: Lerp + Clone + Any,
    {
        Self::new(cx, TweenAnimation::new(initial_value, options))
    }
}

impl<T: Any> AnimatedVar<T> {
    pub fn set_target<'cx>(&self, cx: &mut impl CanWrite<'cx>, value: T) {
        let mut write_context = cx.write_context();
        let node = write_context.get_node_mut(self.id);
        let changed = match &mut node.node_type {
            NodeType::Animation(anim) => anim.inner.set_target_dyn(&value),
            _ => unreachable!(),
        };
        if changed {
            write_context.request_animation(self.id);
        }
    }

    pub fn set_target_and_value<'cx>(&self, cx: &mut impl CanWrite<'cx>, value: T) {
        let mut cx = cx.write_context();
        let node = cx.get_node_mut(self.id);
        match &mut node.node_type {
            NodeType::Animation(anim) => anim.inner.set_target_and_value_dyn(&value),
            _ => unreachable!(),
        };
        cx.notify(self.id);
    }
}

impl<T: 'static> From<AnimatedVar<T>> for ReadSignal<T> {
    fn from(value: AnimatedVar<T>) -> Self {
        ReadSignal::from_node(value.id)
    }
}

impl<T: 'static> From<AnimatedVar<T>> for ViewProp<T> {
    fn from(value: AnimatedVar<T>) -> Self {
        Self::ReadSignal(ReadSignal::from_node(value.id))
    }
}

impl<T: 'static> ReactiveValue for AnimatedVar<T> {
    type Value = T;

    fn track<'cx>(&self, cx: &mut impl CanRead<'cx>) {
        cx.read_context().track(self.id);
    }

    fn with_ref<'cx, R>(&self, cx: &mut impl CanRead<'cx>, f: impl FnOnce(&T) -> R) -> R {
        let mut cx = cx.read_context();
        cx.track(self.id);
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }

    fn with_ref_untracked<'cx, R>(
        &self,
        cx: &mut impl CanRead<'cx>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(get_animation_value_ref(cx.read_context(), self.id)
            .downcast_ref()
            .expect("Animation value had wrong type"))
    }

    fn watch<'cx, F>(self, cx: &mut impl CanCreate<'cx>, f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node(cx.create_context(), self.id, f)
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
    pub fn new<'cx, A: Animation<Value = T> + 'static>(
        cx: &mut impl CanCreate<'cx>,
        f_value: impl Fn(&mut ReadContext) -> T + 'static,
        f_anim: impl FnOnce(T) -> A,
    ) -> Self {
        let id = cx
            .create_context()
            .create_derived_animation_node(move |cx, id| {
                let value = f_value(&mut cx.read_context().with_read_scope(ReadScope::Node(id)));
                let inner = Box::new(f_anim(value));
                let reset_fn = Box::new(
                    move |animation: &mut dyn AnyAnimation,
                          reactive_graph: &mut ReactiveGraph,
                          widgets: &Widgets| {
                        let mut cx = ReadContext {
                            widgets,
                            reactive_graph,
                            scope: ReadScope::Node(id),
                        };
                        animation.set_target_dyn(&f_value(&mut cx))
                    },
                );
                DerivedAnimationState { inner, reset_fn }
            });

        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn spring<'cx>(
        cx: &mut impl CanCreate<'cx>,
        f: impl Fn(&mut ReadContext) -> T + 'static,
        options: SpringOptions,
    ) -> Self
    where
        T: SpringPhysics + Clone + Any,
    {
        Self::new(cx, f, move |initial_value| {
            SpringAnimation::new(initial_value, options)
        })
    }

    pub fn tween<'cx>(
        cx: &mut impl CanCreate<'cx>,
        f: impl Fn(&mut ReadContext) -> T + 'static,
        options: TweenOptions,
    ) -> Self
    where
        T: Lerp + Clone + Any,
    {
        Self::new(cx, f, move |initial_value| {
            TweenAnimation::new(initial_value, options)
        })
    }
}

impl<T: 'static> From<Animated<T>> for ReadSignal<T> {
    fn from(value: Animated<T>) -> Self {
        ReadSignal::from_node(value.id)
    }
}

impl<T> From<Animated<T>> for ViewProp<T> {
    fn from(value: Animated<T>) -> Self {
        Self::ReadSignal(ReadSignal::from_node(value.id))
    }
}

impl<T: 'static> ReactiveValue for Animated<T> {
    type Value = T;

    fn track<'s>(&self, cx: &mut impl CanRead<'s>) {
        cx.read_context().track(self.id);
    }

    fn with_ref<'s, R>(&self, cx: &mut impl CanRead<'s>, f: impl FnOnce(&T) -> R) -> R {
        let mut cx = cx.read_context();
        cx.track(self.id);
        f(get_animation_value_ref(cx, self.id)
            .downcast_ref()
            .expect("AnimationFn value had wrong type"))
    }

    fn with_ref_untracked<'s, R>(
        &self,
        cx: &mut impl CanRead<'s>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        f(get_animation_value_ref(cx.read_context(), self.id)
            .downcast_ref()
            .expect("AnimationFn value had wrong type"))
    }

    fn watch<'s, F>(self, cx: &mut impl CanCreate<'s>, f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        Effect::watch_node::<T>(cx.create_context(), self.id, f)
    }
}

fn get_animation_value_ref(cx: ReadContext<'_>, node_id: NodeId) -> &dyn Any {
    match &cx.reactive_graph.get_node(node_id).node_type {
        NodeType::Animation(animation) => animation.inner.value_dyn(),
        NodeType::DerivedAnimation(animation) => animation.inner.value_dyn(),
        _ => unreachable!(),
    }
}
