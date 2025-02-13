use std::{any::Any, marker::PhantomData};

use super::{accessor::SourceId, CreateContext, NodeId, NodeType, ReadContext, Readable, WriteContext};

pub struct Signal<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<*mut T>
}

impl<T> Clone for Signal<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Signal<T> {}

impl<T: Any> Signal<T> {
    pub fn new(cx: &mut dyn CreateContext, value: T) -> Self {
        let state = SignalState::new(value);
        let owner = cx.owner();
        let id = cx.runtime_mut().create_signal_node(state, owner);
        Self {
            id,
            _marker: PhantomData
        }
    }

	pub fn new_with(cx: &mut dyn CreateContext, f: impl FnOnce(&mut dyn CreateContext) -> T) -> Self {
		let value = f(cx);
		Self::new(cx, value)
	}

    /// Set the current value, notifies subscribers
    pub fn set(&self, cx: &mut impl WriteContext, value: T) {
        self.update(cx, move |val| *val = value)
    }

    /// Set the current value, notifies subscribers
    pub fn set_with(&self, cx: &mut dyn WriteContext, f: impl FnOnce(&mut dyn CreateContext) -> T) {
		let new_value = f(&mut cx.as_create_context(super::Owner::Node(self.id)));
		self.update(cx, move |value| *value = new_value);
    }

    /// Set the current value, notifies subscribers
    pub fn update(&self, cx: &mut dyn WriteContext, f: impl FnOnce(&mut T)) {
		{
            let signal = cx.runtime_mut().get_node_mut(self.id);
            match &mut signal.node_type {
                NodeType::Signal(signal) => {
                    let mut value = signal.value.downcast_mut().expect("Invalid signal value type");
                    f(&mut value);
                },
                _ => unreachable!()
            }
        }
		cx.runtime_mut().notify(self.id);
    }

    pub fn as_read_signal(self) -> ReadSignal<T> {
        ReadSignal {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T: Any> Signal<Vec<T>> {
	pub fn push(&self, cx: &mut impl WriteContext, val: T) {
		self.update(cx, move |value| value.push(val));
	}
}

impl<T: 'static> Readable for Signal<T> {
    type Value = T;

	fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        let scope = cx.scope();
		cx.runtime_mut().track(self.id, scope);
        let value = match &cx.runtime().get_node(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!()
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
    }
}

pub struct ReadSignal<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<*const T>
}

impl<T> Clone for ReadSignal<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for ReadSignal<T> {}

impl<T: 'static> Readable for ReadSignal<T> {
    type Value = T;

	fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&T) -> R) -> R {
        let scope = cx.scope();
		cx.runtime_mut().track(self.id, scope);
        let value = match &cx.runtime().get_node(self.id).node_type {
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
