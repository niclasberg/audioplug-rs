use std::marker::PhantomData;

use super::{CreateContext, NodeId, ReadContext, WriteContext};

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