use std::{any::Any, marker::PhantomData};

use super::{reactive_graph::{RefCountMap, WeakRefCountMap}, NodeId, SignalContext, SignalGet, SignalSet, SignalUpdate};

pub struct Signal<T> {
    pub(super) id: NodeId,
    ref_count_map: WeakRefCountMap,
    _marker: PhantomData<T>
}

impl<T: Any> Signal<T> {
    pub(super) fn new(id: NodeId, ref_count_map: WeakRefCountMap) -> Self {
        Self {
            id,
            ref_count_map,
            _marker: PhantomData
        }
    }

    pub fn update(&self, cx: &mut dyn SignalContext, f: impl Fn(&T) -> T) {
        let new_value = self.with_ref_untracked(cx, f);
        self.set(cx, new_value);
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        RefCountMap::increment_ref_count(&self.ref_count_map, self.id);
        Self { 
            id: self.id, 
            ref_count_map: self.ref_count_map.clone(), 
            _marker: PhantomData 
        }
    }
}

impl<T> Drop for Signal<T> {
    fn drop(&mut self) {
        RefCountMap::decrement_ref_count(&self.ref_count_map, self.id);
    }
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, cx: &mut dyn SignalContext, f: impl FnOnce() -> Self::Value) {
        //cx.set_signal_value(self, f())
        todo!()
    }
}

impl<T: Any> SignalUpdate for Signal<T> {
    type Value = T;

    fn update(&self, cx: &mut dyn SignalContext, f: impl FnOnce(&mut Self::Value)) {
        //cx.update_signal_value(self, f)
        todo!()
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

    fn with_ref<R>(&self, cx: &mut dyn SignalContext, f: impl FnOnce(&T) -> R) -> R {
        //f(cx.get_signal_value_ref(self))
        todo!()
    }

    fn with_ref_untracked<R>(&self, cx: &dyn SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        //f(cx.get_signal_value_ref_untracked(self))
        todo!()
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