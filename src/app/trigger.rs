use std::{any::Any, marker::PhantomData};

use super::{accessor::{MappedAccessor, SourceId}, CreateContext, NodeId, Owner, ReactiveContext, ReadContext, SignalGet, WriteContext};

#[derive(Clone, Copy)]
pub struct Trigger {
	node_id: NodeId,
	_marker: PhantomData<*const ()>,
}

impl Trigger {
	pub fn new(cx: &mut impl CreateContext) -> Self {
		let owner = cx.owner();
		Self {
			node_id: cx.runtime_mut().create_trigger(owner),
			_marker: PhantomData
		}
	}

	pub(super) fn from_node_id(node_id: NodeId) -> Self {
		Self {
			node_id,
			_marker: PhantomData
		}
	}

	pub fn track(&self, cx: &mut dyn ReadContext) {
		let scope = cx.scope().clone();
		cx.runtime_mut().track(self.node_id, scope);
	}

	pub fn trigger(&self, cx: &mut dyn WriteContext) {
		cx.runtime_mut().notify(self.node_id);
	}
}

#[derive(Copy, Clone)]
pub struct MappedValueTrigger<S, T, R, F> {
    source_signal: S,
    f: F,
	trigger_id: NodeId,
    _marker: PhantomData<fn(&T) -> R>
}

impl<S, T, R, F> MappedValueTrigger<S, T, R, F> 
where 
	S: SignalGet<Value = T>,
	F: Fn(&T) -> R
{
	pub fn new(cx: &mut impl CreateContext, source_signal: S, f: F, owner: Option<Owner>) -> Self {
		let trigger_id = cx.runtime_mut().create_trigger(owner);
		Self {
			source_signal,
			f,
			trigger_id,
			_marker: PhantomData
		}
	}

	pub fn dispose(self, cx: &mut impl ReactiveContext) {
		cx.runtime_mut().remove_node(self.trigger_id);
	}
}

impl<S, T, B, F> SignalGet for MappedValueTrigger<S, T, B, F> 
where
    S: SignalGet<Value = T> + 'static,
    T: Any,
    B: Any,
    F: Fn(&T) -> B + 'static
{
	type Value = B;

	fn get_source_id(&self) -> SourceId {
		SourceId::Node(self.trigger_id)
	}

	fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
		f(&self.source_signal.with_ref(cx, |x| (self.f)(x)))
	}
}

impl<S, T, B, F> MappedAccessor<B> for MappedValueTrigger<S, T, B, F> 
where
    S: SignalGet<Value = T> + Clone + 'static,
    T: Any + Clone,
    B: Any + Clone,
    F: Fn(&T) -> B + Clone + 'static
{
    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.trigger_id)
    }

    fn evaluate(&self, ctx: &mut dyn ReadContext) -> B {
        self.source_signal.with_ref(ctx, &self.f)
    }
}
