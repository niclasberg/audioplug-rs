use std::rc::Rc;

use crate::ui::{
    prelude::CanCreate,
    reactive::{ReadContext, WatchContext},
};

use super::{CanRead, Effect, ReactiveValue, ReadScope};

type ComputedFn<T> = dyn Fn(ReadContext) -> T;

#[derive(Clone)]
pub struct Computed<T> {
    f: Rc<ComputedFn<T>>,
}

impl<T> Computed<T> {
    pub fn new(f: impl Fn(ReadContext) -> T + 'static) -> Self {
        Self { f: Rc::new(f) }
    }
}

impl<T: 'static> ReactiveValue for Computed<T> {
    type Value = T;

    fn track<'s>(&self, cx: impl CanRead<'s>) {
        // Only way to track the variables that `f`reads is to run the function
        (self.f)(cx.read_context());
    }

    fn with_ref<'s, R>(&self, cx: impl CanRead<'s>, f: impl FnOnce(&Self::Value) -> R) -> R {
        let value = (self.f)(cx.read_context());
        f(&value)
    }

    fn get<'s>(&self, cx: impl CanRead<'s>) -> Self::Value {
        (self.f)(cx.read_context())
    }

    fn with_ref_untracked<'s, R>(
        &self,
        cx: impl CanRead<'s>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        let value = cx
            .read_context()
            .with_read_scope(ReadScope::Untracked, |cx| (self.f)(cx));
        f(&value)
    }

    fn get_untracked<'s>(&self, cx: impl CanRead<'s>) -> Self::Value {
        cx.read_context()
            .with_read_scope(ReadScope::Untracked, |cx| (self.f)(cx))
    }

    fn watch<'s, F>(self, cx: impl CanCreate<'s>, mut f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        Effect::watch(
            cx,
            move |cx| (self.f)(cx.read_context()),
            move |cx, value, _| {
                f(cx, value);
            },
        )
    }
}
