use std::{any::Any, marker::PhantomData};

use crate::param::{AnyParameter, NormalizedValue, Parameter, ParameterId, ParameterInfo, PlainValue};

use super::{accessor::SourceId, HostHandle, SignalGet, ReactiveContext};

pub trait ParamContext: ReactiveContext {
	fn host_handle(&self) -> &dyn HostHandle;
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
		ctx.get_parameter_ref(self.id).info()
	}

	pub fn begin_edit(&self, ctx: &mut impl ParamContext) {
		ctx.host_handle().begin_edit(self.id);
	}

	pub fn set_value_normalized(&self, ctx: &mut impl ParamContext, value: NormalizedValue) {
		let param_ref = ctx.get_parameter_ref_untracked(self.id);
		param_ref.internal_set_value_normalized(value);
		let info = param_ref.info();
		ctx.host_handle().perform_edit(info, value);
	}

	pub fn set_value_plain(&self, ctx: &mut impl ParamContext, value: PlainValue) {
		let param_ref = ctx.get_parameter_ref_untracked(self.id);
		param_ref.internal_set_value_plain(value);
		let info = param_ref.info();
		let value = info.normalize(value);
		ctx.host_handle().perform_edit(info, value);
	}

	pub fn end_edit(&self, ctx: &mut impl ParamContext) {
		ctx.host_handle().end_edit(self.id);
	}
}

#[derive(Clone, Copy)]
pub struct ParamSignal<T> {
	pub(super) id: ParameterId,
	_phantom: PhantomData<T>
}

impl<T> ParamSignal<T> {
	pub fn new(p: &impl Parameter<T>) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}
}

impl ParamSignal<PlainValue> {
	pub fn new_plain(p: &impl AnyParameter) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}
}

impl ParamSignal<NormalizedValue> {
	pub fn new_normalized(p: &impl AnyParameter) -> Self {
		Self {
			id: p.info().id(),
			_phantom: PhantomData
		}
	}
}

impl<T: Any> SignalGet for ParamSignal<T> {
	type Value = T;

	fn get_source_id(&self) -> SourceId {
		SourceId::Parameter(self.id)
	}

	fn with_ref<R>(&self, cx: &mut dyn ReactiveContext, f: impl FnOnce(&Self::Value) -> R) -> R {
		let param_ref = cx.get_parameter_ref(self.id);
		let value = param_ref.value_as().unwrap();
		f(&value)
	}
	
	fn get(&self, cx: &mut dyn ReactiveContext) -> Self::Value where Self::Value: Clone {
		let param_ref = cx.get_parameter_ref_untracked(self.id);
		param_ref.value_as().unwrap()
	}
}
