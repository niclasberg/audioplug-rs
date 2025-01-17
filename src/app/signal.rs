use std::{any::Any, marker::PhantomData};

use super::{accessor::SourceId, NodeId, NodeType, Path, ReactiveContext, SignalCreator, SignalGet, Trigger};

pub trait SignalContext: ReactiveContext {
	fn notify(&mut self, node_id: NodeId);
	fn get_or_insert_field_trigger(&mut self, node_id: NodeId, path: Path) -> Trigger;
}

pub struct Signal<T> {
    pub(super) id: NodeId,
    _marker: PhantomData<*mut T>
}

impl<T> Clone for Signal<T> {
	fn clone(&self) -> Self {
		Self { id: self.id.clone(), _marker: self._marker.clone() }
	}
}

impl<T> Copy for Signal<T> {}

impl<T: Any> Signal<T> {
    pub fn new(cx: &mut dyn SignalCreator, value: T) -> Self {
        let state = SignalState::new(value);
        let id = cx.create_signal_node(state);
        Self {
            id,
            _marker: PhantomData
        }
    }

	pub fn new_with(cx: &mut dyn SignalCreator, f: impl FnOnce(&mut dyn SignalCreator) -> T) -> Self {
		todo!()
	}

    /// Set the current value, notifies subscribers
    pub fn set(&self, cx: &mut impl SignalContext, value: T) {
        self.update(cx, move |val| *val = value)
    }

    /// Set the current value, notifies subscribers
    pub fn set_with(&self, cx: &mut impl SignalContext, f: impl FnOnce() -> T) {
		self.update(cx, move |value| *value = f());
    }

    /// Set the current value, notifies subscribers
    pub fn update(&self, cx: &mut impl SignalContext, f: impl FnOnce(&mut T)) {
		{
            let signal = cx.get_node_mut(self.id);
            match &mut signal.node_type {
                NodeType::Signal(signal) => {
                    let mut value = signal.value.downcast_mut().expect("Invalid signal value type");
                    f(&mut value);
                },
                _ => unreachable!()
            }
        }
		cx.notify(self.id);
    }
}

impl<T: Any> Signal<Vec<T>> {
	pub fn push(&self, cx: &mut impl SignalContext, val: T) {
		self.update(cx, move |value| value.push(val));
	}
}

impl<T: 'static> SignalGet for Signal<T> {
    type Value = T;

	fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReactiveContext, f: impl FnOnce(&T) -> R) -> R {
		cx.track(self.id);
        let value = match &cx.get_node_mut(self.id).node_type {
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
