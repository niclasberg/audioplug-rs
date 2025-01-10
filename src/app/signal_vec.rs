use std::{any::Any, marker::PhantomData};

use super::{NodeId, SignalGet};

pub trait SignalVecContext {

}

pub struct SignalVec<T> {
    id: NodeId,
    _phantom: PhantomData<*mut T>
}

impl<T: Any> SignalVec<T> {
    pub fn new() -> Self {
        let state = SignalVecState::new::<T>(Vec::new());
        todo!()
    }

    pub fn push(cx: &mut impl SignalVecContext, val: T) {

    }
}

pub struct SignalVecElem<T> {
    id: NodeId,
    index: usize,
    _phantom1: PhantomData<*mut T>
}

impl<T: Any> SignalGet for SignalVecElem<T> {
    type Value = T;

    fn get_source_id(&self) -> super::accessor::SourceId {
        todo!()
    }

    fn with_ref<R>(&self, cx: &mut dyn super::ReactiveContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        //cx.get_node_mut(self.id, child_path)
        todo!()
    }
}

trait InnerState {

}

impl<T> InnerState for Vec<T> {

}

pub struct SignalVecState {
    pub(super) inner: Box<dyn InnerState>
}

impl SignalVecState {
    pub fn new<T: Any>(values: Vec<T>) -> Self {
        Self {
            inner: Box::new(values)
        }
    }
}
