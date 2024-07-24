use std::cell::Cell;

use super::{NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

pub struct BoolParameter {
    info: BoolParameterInfo,
    value: Cell<bool>,
}

impl BoolParameter {
	pub fn new(id: ParameterId, name: &'static str, default: bool) -> Self {
		let info = BoolParameterInfo::new(id, name, default);
		Self {
			info,
			value: Cell::new(default)
		}
	}
}

impl Parameter<bool> for BoolParameter {
    type Info = BoolParameterInfo;

    fn info(&self) -> &Self::Info {
        &self.info
    }

    fn plain_value(&self) -> PlainValue {
        PlainValue::from_bool(self.value.get())
    }

    fn normalized_value(&self) -> NormalizedValue {
        NormalizedValue::from_bool(self.value.get())
    }
	
	fn as_param_ref(&self) -> ParamRef {
		ParamRef::Bool(&self)
	}
    
    fn set_value_normalized(&self, value: NormalizedValue) {
        self.value.replace(value.into());
    }
	
	fn value(&self) -> bool {
		self.value.get()
	}
	
	fn set_value(&self, value: bool) {
		self.value.replace(value);
	}
}


pub struct BoolParameterInfo {
    id: ParameterId,
    name: &'static str,
    default: bool
}

impl BoolParameterInfo {
    pub fn new(id: ParameterId, name: &'static str, default: bool) -> Self {
        Self { id, name, default }
    }
}

impl ParameterInfo for BoolParameterInfo {
	fn id(&self) -> ParameterId {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn default_value(&self) -> super::PlainValue {
		let value = if self.default { 1.0 } else { 0.0 };
		PlainValue::new(value)
	}
	
	fn normalize(&self, value: PlainValue) -> NormalizedValue {
		NormalizedValue(value.0)
	}
	
	fn denormalize(&self, value: super::NormalizedValue) -> PlainValue {
		PlainValue(value.0)
	}
	
	fn step_count(&self) -> usize {
		1
	}
}