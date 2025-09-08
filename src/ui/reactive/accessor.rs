use super::{Computed, Effect, ReactiveValue, ReadContext, ReadSignal};
use crate::{
    core::{Brush, Color, LinearGradient},
    ui::{BuildContext, Widget, WidgetMut},
};

/// Represents a value that is either varying over time (a `ReactiveValue`) or a constant
///
/// Commonly used as input to views.
#[derive(Clone)]
pub enum Accessor<T> {
    Const(T),
    ReadSignal(ReadSignal<T>),
    Computed(Computed<T>),
}

impl<T: 'static> Accessor<T> {
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

impl<T: 'static> ReactiveValue for Accessor<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        match self {
            Self::ReadSignal(signal) => signal.track(cx),
            Self::Computed(computed) => computed.track(cx),
            Self::Const(_) => {}
        }
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        match self {
            Self::ReadSignal(signal) => signal.with_ref(cx, f),
            Self::Computed(computed) => computed.with_ref(cx, f),
            Self::Const(value) => f(value),
        }
    }

    fn get(&self, cx: &mut dyn ReadContext) -> T
    where
        T: Clone,
    {
        match self {
            Self::ReadSignal(signal) => signal.get(cx),
            Self::Computed(computed) => computed.get(cx),
            Self::Const(value) => value.clone(),
        }
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        match self {
            Self::ReadSignal(signal) => signal.with_ref_untracked(cx, f),
            Self::Computed(computed) => computed.with_ref_untracked(cx, f),
            Self::Const(value) => f(value),
        }
    }

    fn get_untracked(&self, cx: &mut dyn super::ReactiveContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        match self {
            Accessor::ReadSignal(read_signal) => read_signal.get_untracked(cx),
            Accessor::Computed(computed) => computed.get_untracked(cx),
            Accessor::Const(value) => value.clone(),
        }
    }

    fn watch<F>(self, cx: &mut dyn super::CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn super::WatchContext, &Self::Value) + 'static,
    {
        match self {
            Accessor::ReadSignal(read_signal) => read_signal.watch(cx, f),
            Accessor::Computed(computed) => computed.watch(cx, f),
            Accessor::Const(_) => Effect::new_empty(),
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
