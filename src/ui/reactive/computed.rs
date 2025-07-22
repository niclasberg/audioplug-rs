use std::rc::Rc;

use super::{Effect, LocalReadContext, ReactiveValue, ReadContext};

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

impl<T: 'static> ReactiveValue for Computed<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        // Only way to track the variables that `f`reads is to run the function
        (self.f)(cx);
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let value = (self.f)(cx);
        f(&value)
    }

    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value {
        (self.f)(cx)
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        let value = (self.f)(&mut LocalReadContext::new(
            cx.app_state_mut(),
            super::Scope::Root,
        ));
        f(&value)
    }

    fn get_untracked(&self, cx: &mut dyn super::ReactiveContext) -> Self::Value {
        (self.f)(&mut LocalReadContext::new(
            cx.app_state_mut(),
            super::Scope::Root,
        ))
    }

    fn watch<F>(self, cx: &mut dyn super::CreateContext, mut f: F) -> Effect
    where
        F: FnMut(&mut dyn super::WatchContext, &Self::Value) + 'static,
    {
        Effect::watch(
            cx,
            move |cx| (self.f)(cx),
            move |cx, value, _| {
                f(cx, value);
            },
        )
    }
}
