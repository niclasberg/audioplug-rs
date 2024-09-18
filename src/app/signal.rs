use std::{any::Any, marker::PhantomData};

use super::{accessor::{MappedAccessor, SourceId}, NodeId, SignalContext, SignalGet, SignalGetContext, SignalSet};

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

    pub fn map<R, F: Fn(&T) -> R>(self, f: F) -> MappedSignal<T, R, F> {
        MappedSignal {
            parent: self,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, cx: &mut impl SignalContext, f: impl FnOnce() -> Self::Value) {
        cx.set_signal_value(self, f())
    }

    fn update(&self, _cx: &mut impl SignalContext, _f: impl FnOnce(&mut Self::Value)) {
        //cx.update_signal_value(self, f)
        todo!()
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

    fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&T) -> R) -> R {
        let value = cx.get_signal_value_ref(self.id).downcast_ref().expect("Signal had wrong type");
        f(value)
    }

    fn with_ref_untracked<R>(&self, cx: &dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        let value = cx.get_signal_value_ref_untracked(self.id).downcast_ref().expect("Signal had wrong type");
        f(value)
    }
}

#[derive(Clone, Copy)]
pub struct MappedSignal<T, R, F> {
    parent: Signal<T>,
    f: F,
    _marker: PhantomData<fn(&T) -> R>
}

impl<T, R, F> MappedSignal<T, R, F> 
where
    T: Any,
    F: Fn(&T) -> R
{
    pub fn map<R2, G: Fn(&R) -> R2>(self, g: G) -> MappedSignal<T, R2, impl Fn(&T) -> R2> {
        let f = move |x: &T| g(&(self.f)(x));
        MappedSignal {
            parent: self.parent,
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, B, F> SignalGet for MappedSignal<T, B, F> 
where
    T: Any,
    F: Fn(&T) -> B
{
    type Value = B;

    fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&B) -> R) -> R {
        f(&self.parent.with_ref(cx, |x| (self.f)(x)))
    }

    fn with_ref_untracked<R>(&self, cx: &dyn SignalGetContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        f(&self.parent.with_ref_untracked(cx, |x| (self.f)(x)))
    }
}

impl<T, B, F> MappedAccessor<B> for MappedSignal<T, B, F> 
where
    T: Any + Clone,
    B: Any + Clone,
    F: Fn(&T) -> B + Clone + 'static
{
    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.parent.id)
    }

    fn get_ref(&self, ctx: &mut dyn SignalGetContext) -> B {
        self.parent.with_ref(ctx, &self.f)
    }

    fn get_ref_untracked(&self, ctx: &dyn SignalGetContext) -> B {
        self.parent.with_ref_untracked(ctx, &self.f)
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