use std::{any::Any, marker::PhantomData};

use super::{NodeId, SignalContext, SignalGet, SignalSet, SignalUpdate};

#[derive(Clone, Copy)]
pub struct Signal<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<*mut T>
}

impl<T: Any> Signal<T> {
    pub(super) fn new(id: NodeId) -> Self {
        Self {
            id,
            _marker: PhantomData
        }
    }

    pub fn update(&self, cx: &mut impl SignalContext, f: impl Fn(&T) -> T) {
        let new_value = self.with_ref_untracked(cx, f);
        self.set(cx, new_value);
    }
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, cx: &mut impl SignalContext, f: impl FnOnce() -> Self::Value) {
        cx.set_signal_value(self, f())
    }
}

impl<T: Any> SignalUpdate for Signal<T> {
    type Value = T;

    fn update(&self, _cx: &mut impl SignalContext, _f: impl FnOnce(&mut Self::Value)) {
        //cx.update_signal_value(self, f)
        todo!()
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

    fn with_ref<R>(&self, cx: &mut impl SignalContext, f: impl FnOnce(&T) -> R) -> R {
        f(cx.get_signal_value_ref(self))
    }

    fn with_ref_untracked<R>(&self, cx: &impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(cx.get_signal_value_ref_untracked(self))
    }
}

pub(super) struct SignalState  {
	pub(super) value: Box<dyn Any>
}

impl SignalState {
    pub fn new<T: Any>(value: T) -> Self {
        Self {
            value: Box::new(value)
        }
    }
}