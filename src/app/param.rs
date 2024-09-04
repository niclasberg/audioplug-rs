use std::marker::PhantomData;

use crate::param::{Parameter, ParameterId};

use super::HostHandle;

pub trait ParamContext {
	fn host_handle(&self) -> &dyn HostHandle;
}

#[derive(Clone, Copy)]
pub struct UiParam<T> {
	id: ParameterId,
	_phantom: PhantomData<T>
} 

impl<T> UiParam<T> {
	pub fn new(p: &dyn Parameter<T>) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}

	pub fn begin_edit(&self, ctx: &dyn ParamContext) {
		ctx.host_handle().begin_edit(self.id);
	}

	pub fn set_value(&self, ctx: &dyn ParamContext, value: T) {

	}

	pub fn end_edit(&self, ctx: &dyn ParamContext) {
		ctx.host_handle().end_edit(self.id);
	}
}

