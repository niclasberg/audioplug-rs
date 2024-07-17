use std::{any::Any, marker::PhantomData};

use super::{RefCountMap, WeakRefCountMap, NodeId, AppState, SignalContext, SignalGet};


pub struct Memo<T> {
    pub(super) id: NodeId,
    ref_count_map: WeakRefCountMap,
    _marker: PhantomData<T>
}

impl<T> Memo<T> {
    pub fn new(id: NodeId, ref_count_map: WeakRefCountMap) -> Self {
        Self {
            id,
            ref_count_map,
            _marker: PhantomData
        }
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        RefCountMap::increment_ref_count(&self.ref_count_map, self.id);
        Self { 
            id: self.id, 
            ref_count_map: self.ref_count_map.clone(), 
            _marker: PhantomData 
        }
    }
}

impl<T> Drop for Memo<T> {
    fn drop(&mut self) {
        RefCountMap::decrement_ref_count(&self.ref_count_map, self.id);
    }
}

impl<T: 'static> SignalGet for Memo<T> {
    type Value = T;

    fn with_ref<R>(&self, ctx: &mut impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        //f(ctx.get_memo_value_ref(self))
        todo!()
    }

    fn with_ref_untracked<R>(&self, ctx: &impl SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        //f(ctx.get_memo_value_ref_untracked(self))
        todo!()
    }
}

pub(super) struct MemoState {
	f: Box<dyn Fn(&mut AppState) -> Box<dyn Any>>,
	pub(super) value: Option<Box<dyn Any>>,
}

impl MemoState {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut AppState) -> Box<dyn Any> + 'static
    {
        Self {
            f: Box::new(f),
            value: None
        }
    }
}