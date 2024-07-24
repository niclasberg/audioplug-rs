use std::cell::Cell;

use super::{bool::BoolParameterInfo, NormalizedValue, ParamRef, Parameter, ParameterId, PlainValue};

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

impl Parameter<bool> for ByPassParameter {
	type Info = BoolParameterInfo; 

	fn info(&self) -> &Self::Info {
		&self.info
	}
	
	fn value(&self) -> bool {
		self.value.get()
	}

	fn plain_value(&self) -> PlainValue {
		PlainValue::from_bool(self.value.get())
	}
	
	fn normalized_value(&self) -> NormalizedValue {
		NormalizedValue::from_bool(self.value.get())
	}

	fn as_param_ref(&self) -> ParamRef {
		ParamRef::ByPass(&self)
	}
	
	fn set_value(&self, value: bool) {
		self.value.replace(value);
	}

	fn set_value_normalized(&self, value: NormalizedValue) {
		self.value.replace(value.into());
	}
}