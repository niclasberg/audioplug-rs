use std::marker::PhantomData;

use crate::param::{AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

use super::HostHandle;

pub trait ParamContext {
	fn host_handle(&self) -> &dyn HostHandle;
	fn get_parameter_ref<'a>(&'a self, id: ParameterId) -> Option<ParamRef<'a>>;
	fn get_parameter_as<'a, P: AnyParameter>(&'a self, param: &ParamEditor<P>) -> &'a P;
}

#[derive(Clone, Copy)]
pub struct ParamEditor<P> {
	pub(super) id: ParameterId,
	_phantom: PhantomData<P>
} 

impl<P: AnyParameter> ParamEditor<P> {
	pub fn new(p: &P) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}

	pub fn begin_edit(&self, ctx: &mut impl ParamContext) {
		ctx.host_handle().begin_edit(self.id);
	}

	pub fn set_value_normalized(&self, ctx: &mut impl ParamContext, value: NormalizedValue) {
		ctx.host_handle().perform_edit(self.id, value);
	}

	pub fn set_value_plain(&self, ctx: &mut impl ParamContext, value: PlainValue) {
		let value = ctx.get_parameter_ref(self.id).unwrap().info().normalize(value);
		ctx.host_handle().perform_edit(self.id, value);
	}

	pub fn end_edit(&self, ctx: &mut impl ParamContext) {
		ctx.host_handle().end_edit(self.id);
	}
}

impl<P: Parameter> ParamEditor<P> {
	pub fn set_value(&self, ctx: &mut impl ParamContext, value: P::Value) {
		
	}
}
