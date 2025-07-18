use std::rc::Rc;

use super::{Brush, BuildContext, LinearGradient, ReadContext, Readable, Widget, WidgetMut};
use crate::{
    app::{Effect, ReadSignal},
    core::Color,
};

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

/// Represents a value that is either varying over time (a `Readable`) or a constant
///
/// Commonly used as input to views.
#[derive(Clone)]
pub enum Accessor<T> {
    Const(T),
    ReadSignal(ReadSignal<T>),
    Computed(Computed<T>),
}

impl<T: 'static> Accessor<T> {
    pub fn get(&self, cx: &mut dyn ReadContext) -> T
    where
        T: Clone,
    {
        match self {
            Self::ReadSignal(signal) => signal.get(cx),
            Self::Const(value) => value.clone(),
            Self::Computed(computed) => (computed.f)(cx),
        }
    }

    pub fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::ReadSignal(signal) => signal.with_ref(cx, f),
            Self::Const(value) => f(value),
            Self::Computed(computed) => f(&(computed.f)(cx)),
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
            Self::ReadSignal(signal) => {
                signal.watch(cx, move |cx, value| {
                    f(f_map(value), cx.widget_mut(widget_id))
                });
            }
            Self::Computed(computed) => {
                let value_fn = computed.f.clone();
                Effect::watch(
                    cx,
                    move |cx| value_fn(cx),
                    move |cx, value, _| {
                        f(f_map(value), cx.widget_mut(widget_id));
                    },
                );
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
