use std::{any::Any, marker::PhantomData};

use super::{accessor::SourceId, NodeId, NodeType, Path, ReactiveContext, SignalCreator, SignalGet};

pub trait SignalContext: ReactiveContext {
    fn update_signal_value<T: Any>(&mut self, signal: &Signal<T>, f: impl FnOnce(&mut T));
}

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

    /// Set the current value, notifies subscribers
    pub fn set(&self, cx: &mut impl SignalContext, value: T) {
        self.set_with(cx, move || value)
    }

    /// Set the current value, notifies subscribers
    pub fn set_with(&self, cx: &mut impl SignalContext, f: impl FnOnce() -> T) {
        cx.update_signal_value(self, move |value| {
            *value = f();
        });
    }

    /// Set the current value, notifies subscribers
    pub fn update(&self, cx: &mut impl SignalContext, f: impl FnOnce(&mut T)) {
        cx.update_signal_value(self, f);
    }
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

	fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReactiveContext, f: impl FnOnce(&T) -> R) -> R {
        let value = match &cx.get_node_ref(self.id, Path::ROOT).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!()
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
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
