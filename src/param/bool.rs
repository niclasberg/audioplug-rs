use std::{any::Any, cell::Cell};

use super::{AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, ParseError, PlainValue};

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

impl AnyParameter for BoolParameter {
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

impl Parameter for BoolParameter {
	type Value = bool;

	fn as_param_ref(&self) -> ParamRef {
		ParamRef::Bool(&self)
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
	
	fn value(&self) -> bool {
		self.value.get()
	}
	
	fn set_value(&self, value: bool) {
		self.value.replace(value);
	}
}

#[derive(Clone, Copy)]
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
	
	fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError> {
		if str.eq_ignore_ascii_case("on") { 
			Ok(NormalizedValue::from_bool(true)) 
		} else if str.eq_ignore_ascii_case("off") {
			Ok(NormalizedValue::from_bool(false)) 
		} else {
			Err(ParseError)
		}
	}
	
	fn string_from_value(&self, value: NormalizedValue) -> String {
		if value.into() {
			"On".to_string()
		} else {
			"Off".to_string()
		}
	}
}