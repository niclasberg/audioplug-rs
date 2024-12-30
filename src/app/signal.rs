use std::{any::Any, marker::PhantomData};

use super::{accessor::SourceId, NodeId, SignalContext, SignalCreator, SignalGet, SignalGetContext, SignalSet};

#[derive(Clone, Copy)]
pub struct Signal<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<*mut T>
}

impl<T: Any> Signal<T> {
    pub fn new(cx: &mut impl SignalCreator, value: T) -> Self {
        let state = SignalState::new(value);
        let id = cx.create_signal_node(state);
        Self {
            id,
            _marker: PhantomData
        }
    }
}

impl<T: Any> SignalSet for Signal<T> {
    type Value = T;

    fn set_with(&self, cx: &mut impl SignalContext, f: impl FnOnce() -> Self::Value) {
        cx.set_signal_value(self, f())
    }

    fn update(&self, cx: &mut impl SignalContext, f: impl FnOnce(&mut Self::Value)) {
        //let new_value = self.with_ref(cx, f);
        //self.set(cx, new_value);
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

	fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn SignalGetContext, f: impl FnOnce(&T) -> R) -> R {
        let value = cx.get_node_value_ref(self.id).downcast_ref().expect("Signal had wrong type");
        f(value)
    }
}

pub struct SignalState  {
	pub(super) value: Box<dyn Any>
}

impl SignalState {
    pub fn new<T: Any>(value: T) -> Self {
        Self {
            value: Box::new(value)
        }
    }
}

pub struct VecSignal<T> {
    _phantom: PhantomData<*mut T>
}

impl<T> VecSignal<T> {

}