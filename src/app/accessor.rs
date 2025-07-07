use std::rc::Rc;

use super::{
    effect::BindingState, signal::ReadSignal, Animated, AnimatedFn, Brush, BuildContext,
    EffectState, LinearGradient, Memo, NodeId, Owner, ParamSignal, ReactiveContext, ReadContext,
    Readable, Signal, Widget, WidgetMut,
};
use crate::{app::Effect, core::Color, param::ParameterId};

pub trait MappedAccessor<T> {
    fn get_source_id(&self) -> SourceId;
    fn evaluate(&self, ctx: &mut dyn ReadContext) -> T;
    fn evaluate_untracked(&self, ctx: &mut dyn ReactiveContext) -> T;
}

#[derive(Clone, Copy)]
pub enum SourceId {
    Parameter(ParameterId),
    Node(NodeId),
}

type ComputedFn<T> = dyn Fn(&mut dyn ReadContext) -> T;

#[derive(Clone)]
pub struct Computed<T> {
    f: Rc<ComputedFn<T>>,
}

impl<T> Computed<T> {
    pub fn new(f: impl Fn(&mut dyn ReadContext) -> T + 'static) -> Self {
        Self { f: Rc::new(f) }
    }
}

#[derive(Clone)]
pub enum Accessor<T> {
    Signal(Signal<T>),
    ReadSignal(ReadSignal<T>),
    Memo(Memo<T>),
    Parameter(ParamSignal<T>),
    Const(T),
    Mapped(Rc<dyn MappedAccessor<T>>),
    Computed(Computed<T>),
    Animated(Animated<T>),
    AnimatedFn(AnimatedFn<T>),
}

impl<T: 'static> Accessor<T> {
    pub fn get(&self, cx: &mut dyn ReadContext) -> T
    where
        T: Clone,
    {
        match self {
            Self::Signal(signal) => signal.get(cx),
            Self::ReadSignal(signal) => signal.get(cx),
            Self::Memo(memo) => memo.get(cx),
            Self::Const(value) => value.clone(),
            Self::Parameter(param) => param.get(cx),
            Self::Mapped(mapped) => mapped.evaluate(cx),
            Self::Computed(computed) => (computed.f)(cx),
            Self::Animated(animated) => animated.get(cx),
            Self::AnimatedFn(animated) => animated.get(cx),
        }
    }

    pub fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::Signal(signal) => signal.with_ref(cx, f),
            Self::ReadSignal(signal) => signal.with_ref(cx, f),
            Self::Memo(memo) => memo.with_ref(cx, f),
            Self::Const(value) => f(value),
            Self::Parameter(param) => param.with_ref(cx, f),
            Self::Mapped(mapped) => f(&mapped.evaluate(cx)),
            Self::Computed(computed) => f(&(computed.f)(cx)),
            Self::Animated(animated) => animated.with_ref(cx, f),
            Self::AnimatedFn(animated) => animated.with_ref(cx, f),
        }
    }

    pub fn get_and_bind<W: Widget + ?Sized>(
        self,
        cx: &mut BuildContext<W>,
        f: impl Fn(T, WidgetMut<'_, W>) + 'static,
    ) -> T
    where
        T: Clone,
    {
        self.get_and_bind_mapped(cx, T::clone, f)
    }

    pub fn get_and_bind_mapped<W: Widget + ?Sized, U: 'static>(
        self,
        cx: &mut BuildContext<W>,
        f_map: fn(&T) -> U,
        f: impl Fn(U, WidgetMut<'_, W>) + 'static,
    ) -> U {
        let value = self.with_ref(cx, f_map);
        self.bind_mapped(cx, f_map, f);
        value
    }

    pub fn bind<W: Widget + ?Sized>(
        self,
        cx: &mut BuildContext<W>,
        f: impl Fn(T, WidgetMut<'_, W>) + 'static,
    ) where
        T: Clone,
    {
        self.bind_mapped(cx, T::clone, f);
    }

    pub fn bind_mapped<W: Widget + ?Sized, U: 'static, F>(
        self,
        cx: &mut BuildContext<W>,
        f_map: fn(&T) -> U,
        f: F,
    ) where
        F: Fn(U, WidgetMut<'_, W>) + 'static,
    {
        let widget_id = cx.id();
        match self {
            Self::Signal(signal) => {
                Effect::watch(cx, signal, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::ReadSignal(signal) => {
                Effect::watch(cx, signal, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::Memo(memo) => {
                Effect::watch(cx, memo, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::Parameter(param) => {
                Effect::watch(cx, param, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::Animated(animated) => {
                Effect::watch(cx, animated, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::AnimatedFn(animated) => {
                Effect::watch(cx, animated, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::Mapped(ref mapped) => {
                let source_id = mapped.get_source_id();
                let state = BindingState::new(move |app_state| {
                    let value = self.with_ref(app_state, f_map);
                    let node = WidgetMut::new(app_state, widget_id.id);
                    f(value, node);
                });

                cx.runtime_mut().create_binding_node(
                    source_id,
                    state,
                    Some(Owner::Widget(widget_id.id)),
                );
            }
            Self::Computed(computed) => {
                let value_fn = computed.f.clone();
                let state = EffectState::new(move |cx| {
                    let value = f_map(&value_fn(cx));
                    let widget = cx.widget_mut(widget_id);
                    f(value, widget);
                });
                cx.runtime_mut()
                    .create_effect_node(state, Some(Owner::Widget(widget_id.id)), true);
            }
            Accessor::Const(_) => {}
        }
    }
}

impl<T: Default> Default for Accessor<T> {
    fn default() -> Self {
        Self::Const(T::default())
    }
}

impl<T> From<T> for Accessor<T> {
    fn from(value: T) -> Self {
        Self::Const(value)
    }
}

impl From<&str> for Accessor<String> {
    fn from(value: &str) -> Self {
        Self::Const(value.to_string())
    }
}

impl From<Color> for Accessor<Brush> {
    fn from(value: Color) -> Self {
        Self::Const(value.into())
    }
}

impl From<LinearGradient> for Accessor<Brush> {
    fn from(value: LinearGradient) -> Self {
        Self::Const(value.into())
    }
}

impl<T> From<Computed<T>> for Accessor<T> {
    fn from(value: Computed<T>) -> Self {
        Self::Computed(value)
    }
}
