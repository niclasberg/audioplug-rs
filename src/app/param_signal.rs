use std::marker::PhantomData;

use crate::param::ParameterId;
use super::NodeId;

pub trait ParamSignalContext {
	
}

pub struct ParamSignal<T> {
	node_id: NodeId,
	parameter_id: ParameterId,
	_phantom: PhantomData<T>
} 

impl<T> ParamSignal<T> {
	pub fn begin_edit() {

	}

	pub fn end_edit() {

	}
}

