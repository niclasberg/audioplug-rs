use std::rc::Rc;

use crate::ui::{
    Prop,
    prelude::CanCreate,
    reactive::{EffectContext, ReadContext},
};

use super::{CanRead, Effect, ReactiveValue, ReadScope};

type ComputedFn<T> = dyn Fn(&mut ReadContext) -> T;

#[derive(Clone)]
enum ComputedInner<T> {
    Fn(Rc<ComputedFn<T>>),
    ConstFn(fn(&mut ReadContext) -> T),
}

#[derive(Clone)]
pub struct Computed<T> {
    inner: ComputedInner<T>,
}

impl<T> Computed<T> {
    pub fn new(f: impl Fn(&mut ReadContext) -> T + 'static) -> Self {
        Self {
            inner: ComputedInner::Fn(Rc::new(f)),
        }
    }

    pub const fn new_const(f: fn(&mut ReadContext) -> T) -> Self {
        Self {
            inner: ComputedInner::ConstFn(f),
        }
    }

    fn eval(&self, cx: &mut ReadContext) -> T {
        match &self.inner {
            ComputedInner::Fn(f) => f(cx),
            ComputedInner::ConstFn(f) => f(cx),
        }
    }
}

impl<T> From<Computed<T>> for Prop<T> {
    fn from(value: Computed<T>) -> Self {
        Self::Computed(value)
    }
}

impl<T: 'static> ReactiveValue for Computed<T> {
    type Value = T;

    fn track<'s>(&self, cx: &mut impl CanRead<'s>) {
        // Only way to track the variables that `f`reads is to run the function
        self.eval(&mut cx.read_context());
    }

    fn with_ref<'s, R>(&self, cx: &mut impl CanRead<'s>, f: impl FnOnce(&Self::Value) -> R) -> R {
        let value = self.eval(&mut cx.read_context());
        f(&value)
    }

    fn get<'s>(&self, cx: &mut impl CanRead<'s>) -> Self::Value {
        self.eval(&mut cx.read_context())
    }

    fn with_ref_untracked<'s, R>(
        &self,
        cx: &mut impl CanRead<'s>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        // If we are reading from a tracked scope (for instance reading a Computed in an Effect),
        // we want to ignore this scope while evaluating the Computed. If we didn't do this
        // we would end up tracking everything that is read while evaluating the Computed.
        // I know, this is a bit weird, but required for correct semantics.
        let value = self.eval(&mut cx.read_context().with_read_scope(ReadScope::Untracked));
        f(&value)
    }

    fn get_untracked<'s>(&self, cx: &mut impl CanRead<'s>) -> Self::Value {
        self.eval(&mut cx.read_context().with_read_scope(ReadScope::Untracked))
    }

    fn watch<'s, F>(self, cx: &mut impl CanCreate<'s>, mut f: F) -> Effect
    where
        F: FnMut(&mut EffectContext, &Self::Value) + 'static,
    {
        Effect::watch(
            cx,
            move |cx| self.eval(&mut cx.read_context()),
            move |cx, value, _| {
                f(cx, value);
            },
        )
    }
}
