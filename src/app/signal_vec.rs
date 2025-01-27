use std::{any::Any, cell::RefCell, collections::VecDeque, marker::PhantomData, rc::Rc};

use super::{accessor::SourceId, signal::SignalState, CreateContext, NodeId, NodeType, Owner, Runtime, Readable, Trigger, WriteContext};

#[derive(Copy, Clone)]
pub struct SignalVec<T> {
    id: NodeId,
    _phantom: PhantomData<*mut T>
}

impl<T: Any> SignalVec<T> {
    pub fn new(cx: &mut dyn CreateContext) -> Self {
        let state = SignalState::new(Inner::<T> {
            values: Vec::new(),
            triggers: Vec::new(),
        });
		let owner = cx.owner();
        let id = cx.runtime_mut().create_signal_node(state, owner);
		Self {
			id,
			_phantom: PhantomData
		}
    }

    pub fn push(&self, cx: &mut impl WriteContext, value: T) {
		let trigger = Trigger::new(&mut cx.as_create_context(Owner::Node(self.id)));
		self.with_inner_mut(cx.runtime_mut(), move |inner| {
			inner.values.push(value);
			inner.triggers.push(trigger);
		});
		cx.runtime_mut().notify(self.id);
    }

	pub fn extend(&self, cx: &mut impl WriteContext, iter: impl IntoIterator<Item = T>) {
		self.with_inner_mut(cx.runtime_mut(), move |inner| {
			inner.values.extend(iter);
		});
	}

	fn with_inner<R>(&self, cx: &Runtime, f: impl FnOnce(&Inner<T>) -> R) -> R {
		let value = match &cx.get_node(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_ref(),
            _ => unreachable!()
        };
        f(value.downcast_ref().expect("Signal had wrong type"))
	}

	fn with_inner_mut<R>(&self, cx: &mut Runtime, f: impl FnOnce(&mut Inner<T>) -> R) -> R {
		let value = match &mut cx.get_node_mut(self.id).node_type {
            NodeType::Signal(signal) => signal.value.as_mut(),
            _ => unreachable!()
        };
        f(value.downcast_mut().expect("Signal had wrong type"))
	}
}

impl<T: Any> Readable for SignalVec<T> {
	type Value = Vec<T>;

	fn get_source_id(&self) -> SourceId {
		SourceId::Node(self.id)
	}

	fn with_ref<R>(&self, cx: &mut dyn super::ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
		let scope = cx.scope();
		cx.runtime_mut().track(self.id, scope);
		self.with_inner(cx.runtime(), move |value| f(&value.values))
	}
}

pub struct AtIndex<Parent, T> {
    index: usize,
    parent: Parent,
	id: NodeId,
    _phantom2: PhantomData<*const T>
}

impl<T: Any> Readable for AtIndex<SignalVec<T>, T> {
	type Value = T;

	fn get_source_id(&self) -> SourceId {
		SourceId::Node(self.id)
	}

	fn with_ref<R>(&self, cx: &mut dyn super::ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
		let scope = cx.scope();
		cx.runtime_mut().track(self.id, scope);
		self.parent.with_inner(cx.runtime(), move |inner| f(inner.values.get(self.index).unwrap()))
	}
}

pub struct SignalVecElem<T> {
    parent_id: NodeId,
    index: usize,
    _phantom1: PhantomData<*mut T>
}

impl<T: Any> Readable for SignalVecElem<T> {
    type Value = T;

    fn get_source_id(&self) -> super::accessor::SourceId {
        todo!()
    }

    fn with_ref<R>(&self, cx: &mut dyn super::ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        //cx.get_node_mut(self.id, child_path)
        todo!()
    }
}

struct Inner<T> {
	values: Vec<T>,
	triggers: Vec<Trigger>
}

trait StreamStateInner {

}

impl<T> StreamStateInner for VecDeque<T> {

}

pub struct StreamState {
	queue: Rc<RefCell<dyn StreamStateInner>>
}

