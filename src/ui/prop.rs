use super::reactive::{CanCreate, CanRead, Computed, Effect, ReactiveValue, ReadSignal};
use crate::{
    core::{Color, LinearGradient},
    ui::{BuildContext, Paint, Widget, WidgetMut, reactive::EffectContext, style::Fill},
};

/// Represents a value that is either varying over time (a `ReactiveValue`) or a constant
///
/// Commonly used as input to views.
#[derive(Clone)]
pub enum Prop<T> {
    Const(T),
    ReadSignal(ReadSignal<T>),
    Computed(Computed<T>),
}

impl<T: 'static> Prop<T> {
    pub const fn from_value(value: T) -> Self {
        Self::Const(value)
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
        self.watch(cx, move |cx, value| {
            f(f_map(value), cx.widget_mut(widget_id))
        });
    }
}

impl<T: 'static> ReactiveValue for Prop<T> {
    type Value = T;

    fn track<'a>(&self, cx: &mut impl CanRead<'a>) {
        match self {
            Self::ReadSignal(signal) => signal.track(cx),
            Self::Computed(computed) => computed.track(cx),
            Self::Const(_) => {}
        }
    }

    fn with_ref<'a, R>(&self, cx: &mut impl CanRead<'a>, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Self::ReadSignal(signal) => signal.with_ref(cx, f),
            Self::Computed(computed) => computed.with_ref(cx, f),
            Self::Const(value) => f(value),
        }
    }

    fn get<'a>(&self, cx: &mut impl CanRead<'a>) -> T
    where
        T: Clone,
    {
        match self {
            Self::ReadSignal(signal) => signal.get(cx),
            Self::Computed(computed) => computed.get(cx),
            Self::Const(value) => value.clone(),
        }
    }

    fn with_ref_untracked<'a, R>(
        &self,
        cx: &mut impl CanRead<'a>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        match self {
            Self::ReadSignal(signal) => signal.with_ref_untracked(cx, f),
            Self::Computed(computed) => computed.with_ref_untracked(cx, f),
            Self::Const(value) => f(value),
        }
    }

    fn get_untracked<'a>(&self, cx: &mut impl CanRead<'a>) -> Self::Value
    where
        Self::Value: Clone,
    {
        match self {
            Prop::ReadSignal(read_signal) => read_signal.get_untracked(cx),
            Prop::Computed(computed) => computed.get_untracked(cx),
            Prop::Const(value) => value.clone(),
        }
    }

    fn watch<'a, F>(self, cx: &mut impl CanCreate<'a>, f: F) -> Effect
    where
        F: FnMut(&mut EffectContext, &Self::Value) + 'static,
    {
        match self {
            Prop::ReadSignal(read_signal) => read_signal.watch(cx, f),
            Prop::Computed(computed) => computed.watch(cx, f),
            Prop::Const(_) => Effect::new_empty(),
        }
    }
}

impl<T: Default> Default for Prop<T> {
    fn default() -> Self {
        Self::Const(T::default())
    }
}

impl<T> From<T> for Prop<T> {
    fn from(value: T) -> Self {
        Self::Const(value)
    }
}

impl From<&str> for Prop<String> {
    fn from(value: &str) -> Self {
        Self::Const(value.to_string())
    }
}

impl From<Color> for Prop<Fill> {
    fn from(value: Color) -> Self {
        Self::Const(value.into())
    }
}
