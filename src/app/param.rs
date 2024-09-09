use std::marker::PhantomData;

use crate::param::{AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

use super::{HostHandle, SignalGet};

pub trait ParamContext {
	fn host_handle(&self) -> &dyn HostHandle;
	fn get_parameter_ref<'a>(&'a self, id: ParameterId) -> Option<ParamRef<'a>>;
	fn get_parameter_as<'a, P: AnyParameter>(&'a self, param: &ParamEditor<P>) -> &'a P;
}

pub struct ParamEditor<P> {
	pub(super) id: ParameterId,
	_phantom: PhantomData<P>
} 

impl<P> Clone for ParamEditor<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for ParamEditor<P> {}

impl<P: AnyParameter> ParamEditor<P> {
	pub fn new(p: &P) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}

	pub fn info<'a>(&self, ctx: &'a mut impl ParamContext) -> &'a dyn ParameterInfo {
		ctx.get_parameter_ref(self.id).map(|p| p.info()).unwrap()
	}

	pub fn begin_edit(&self, ctx: &mut impl ParamContext) {
		ctx.host_handle().begin_edit(self.id);
	}

	pub fn set_value_normalized(&self, ctx: &mut impl ParamContext, value: NormalizedValue) {
		let param_ref = ctx.get_parameter_ref(self.id).unwrap();
		let info = param_ref.info();
		ctx.host_handle().perform_edit(info, value);
	}

	pub fn set_value_plain(&self, ctx: &mut impl ParamContext, value: PlainValue) {
		let param_ref = ctx.get_parameter_ref(self.id).unwrap();
		let info = param_ref.info();
		let value = info.normalize(value);
		ctx.host_handle().perform_edit(info, value);
	}

	pub fn end_edit(&self, ctx: &mut impl ParamContext) {
		ctx.host_handle().end_edit(self.id);
	}
}

impl<P: Parameter> ParamEditor<P> {
	pub fn set_value(&self, ctx: &mut impl ParamContext, value: P::Value) {
		
	}
}

#[derive(Clone, Copy)]
pub struct ParamSignal<P: AnyParameter> {
	pub(super) id: ParameterId,
	_phantom: PhantomData<P>
}

impl<P: AnyParameter> ParamSignal<P> {
	pub fn new(p: &P) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}
}

impl<P: AnyParameter> SignalGet for ParamSignal<P> {
	type Value = P;

	fn with_ref<R>(&self, cx: &mut impl super::SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
		f(cx.get_parameter_ref(&self))
	}

	fn with_ref_untracked<R>(&self, cx: &impl super::SignalContext, f: impl FnOnce(&Self::Value) -> R) -> R {
		f(cx.get_parameter_ref_untracked(&self))
	}
}