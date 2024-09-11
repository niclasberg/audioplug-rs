use std::{any::Any, cell::Cell};

use super::{bool::BoolParameterInfo, AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

pub struct ByPassParameter {
	info: BoolParameterInfo,
	value: Cell<bool>,
}

impl ByPassParameter {
	pub fn new(id: ParameterId) -> Self {
		let info = BoolParameterInfo::new(id, "Bypass", false);
		Self {
			info, 
			value: Cell::new(false)
		}
	}
}

impl AnyParameter for ByPassParameter {
	fn info(&self) -> &dyn ParameterInfo {
        &self.info
    }

	fn plain_value(&self) -> PlainValue {
		PlainValue::from_bool(self.value.get())
	}
	
	fn normalized_value(&self) -> NormalizedValue {
		NormalizedValue::from_bool(self.value.get())
	}

	fn set_value_normalized(&self, value: NormalizedValue) {
		self.value.replace(value.into());
	}
}

impl Parameter<bool> for ByPassParameter {
	fn value(&self) -> bool {
		self.value.get()
	}
	
	fn set_value(&self, value: bool) {
		self.value.replace(value);
	}

	fn as_param_ref(&self) -> ParamRef {
		ParamRef::ByPass(&self)
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}