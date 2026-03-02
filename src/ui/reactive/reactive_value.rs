use std::{any::Any, marker::PhantomData};

use super::{CanCreate, CanRead, Computed, Effect, WatchContext};
use crate::ui::ViewProp;

pub trait ReactiveValue: Into<ViewProp<Self::Value>> {
    type Value;

    /// Map the current value using `f` and subscribe to changes
    fn with_ref<'s, R>(&self, cx: impl CanRead<'s>, f: impl FnOnce(&Self::Value) -> R) -> R {
        let mut read_context = cx.read_context();
        let ret = self.with_ref_untracked(&mut read_context, f);
        self.track(read_context);
        ret
    }

    fn track<'s>(&self, cx: impl CanRead<'s>);

    /// Get the current value and subscribe to changes
    fn get<'s>(&self, cx: impl CanRead<'s>) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.with_ref(cx, Self::Value::clone)
    }

    /// Map the current value using `f`
    fn with_ref_untracked<'s, R>(
        &self,
        cx: impl CanRead<'s>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R;

    /// Get the current value
    fn get_untracked<'s>(&self, cx: impl CanRead<'s>) -> Self::Value
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
    fn watch<'s, F>(self, cx: impl CanCreate<'s>, f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static;
}

#[derive(Clone, Copy)]
pub struct Mapped<S, T, R, F> {
    parent: S,
    map_fn: F,
    _marker: PhantomData<fn(&T) -> R>,
}

impl<S, T, R, F> From<Mapped<S, T, R, F>> for ViewProp<R>
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

    fn track<'s>(&self, cx: impl CanRead<'s>) {
        self.parent.track(cx);
    }

    fn with_ref<'s, R2>(&self, cx: impl CanRead<'s>, f: impl FnOnce(&Self::Value) -> R2) -> R2 {
        self.parent.with_ref(cx, |x| f(&(self.map_fn)(x)))
    }

    fn get<'s>(&self, cx: impl CanRead<'s>) -> Self::Value {
        self.parent.with_ref(cx, |x| (self.map_fn)(x))
    }

    fn with_ref_untracked<'s, R2>(
        &self,
        cx: impl CanRead<'s>,
        f: impl FnOnce(&Self::Value) -> R2,
    ) -> R2 {
        self.parent.with_ref_untracked(cx, |x| f(&(self.map_fn)(x)))
    }

    fn get_untracked<'s>(&self, cx: impl CanRead<'s>) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.parent.with_ref_untracked(cx, |x| (self.map_fn)(x))
    }

    fn watch<'s, F2>(self, cx: impl CanCreate<'s>, mut f: F2) -> Effect
    where
        F2: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        self.parent.watch(cx, move |cx, value| {
            let mapped_value = (self.map_fn)(value);
            f(cx, &mapped_value);
        })
    }
}
