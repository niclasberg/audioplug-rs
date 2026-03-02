use std::rc::Rc;

use crate::ui::{
    prelude::CanCreate,
    reactive::{ReadContext, WatchContext},
};

use super::{CanRead, Effect, ReactiveValue, ReadScope};

type ComputedFn<T> = dyn Fn(&mut ReadContext) -> T;

#[derive(Clone)]
pub struct Computed<T> {
    f: Rc<ComputedFn<T>>,
}

impl<T> Computed<T> {
    pub fn new(f: impl Fn(&mut ReadContext) -> T + 'static) -> Self {
        Self { f: Rc::new(f) }
    }
}

impl<T: 'static> ReactiveValue for Computed<T> {
    type Value = T;

    fn track<'s>(&self, cx: &mut impl CanRead<'s>) {
        // Only way to track the variables that `f`reads is to run the function
        (self.f)(&mut cx.read_context());
    }

    fn with_ref<'s, R>(&self, cx: &mut impl CanRead<'s>, f: impl FnOnce(&Self::Value) -> R) -> R {
        let value = (self.f)(&mut cx.read_context());
        f(&value)
    }

    fn get<'s>(&self, cx: &mut impl CanRead<'s>) -> Self::Value {
        (self.f)(&mut cx.read_context())
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
        let value = (self.f)(&mut cx.read_context().with_read_scope(ReadScope::Untracked));
        f(&value)
    }

    fn get_untracked<'s>(&self, cx: &mut impl CanRead<'s>) -> Self::Value {
        (self.f)(&mut cx.read_context().with_read_scope(ReadScope::Untracked))
    }

    fn watch<'s, F>(self, cx: &mut impl CanCreate<'s>, mut f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        Effect::watch(
            cx,
            move |cx| (self.f)(&mut cx.read_context()),
            move |cx, value, _| {
                f(cx, value);
            },
        )
    }
}
