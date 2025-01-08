use std::marker::PhantomData;

use super::{signal::SignalContext, NodeId, ReactiveContext, SignalCreator};

#[derive(Clone, Copy)]
pub struct Trigger {
	node_id: NodeId,
	_marker: PhantomData<*const ()>,
}

impl Trigger {
	pub fn new(cx: &mut impl SignalCreator) -> Self {
		Self {
			node_id: cx.create_trigger(),
			_marker: PhantomData
		}
	}

	pub fn track(&self, cx: &mut dyn ReactiveContext) {
		cx.track(self.node_id);
	}

	pub fn trigger(&self, cx: &mut dyn SignalContext) {
		cx.notify(self.node_id);
	}
}