use std::cell::Cell;

use super::{NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

pub struct StringListParameter {
	index: Cell<usize>,
	info: StringListParameterInfo
}

impl StringListParameter {

}

impl Parameter<usize> for StringListParameter {
	fn info(&self) -> &dyn ParameterInfo {
		&self.info
	}

	fn value(&self) -> usize {
		self.index.get()
	}

	fn plain_value(&self) -> PlainValue {
		PlainValue(self.index.get() as f64)
	}
	
	fn set_value_normalized(&self, value: NormalizedValue) {
		todo!()
	}
	
	fn set_value(&self, value: usize) {
		self.index.replace(value);
	}

	fn as_param_ref(&self) -> ParamRef {
		ParamRef::StringList(&self)
	}
}

pub struct StringListParameterInfo {
	id: ParameterId,
    name: &'static str,
    strings: Vec<String>,
    default_index: usize,
}

impl StringListParameterInfo {
    pub fn new(id: ParameterId, name: &'static str, strings: impl Into<Vec<String>>, default_index: usize) -> Self {
        Self {
			id,
            name, 
            strings: strings.into(),
            default_index
        }
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.strings.iter().position(|x| x == key)
    }
}

impl ParameterInfo for StringListParameterInfo {
	fn id(&self) -> ParameterId {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn default_value(&self) -> PlainValue {
		PlainValue::new(self.default_index as f64)
	}
	
	fn normalize(&self, value: PlainValue) -> NormalizedValue {
		NormalizedValue(value.0.clamp(0.0, self.string_count() as f64) / (self.string_count() as f64))
	}
	
	fn denormalize(&self, value: NormalizedValue) -> PlainValue {
		PlainValue(value.0 * (self.string_count() as f64))
	}
	
	fn step_count(&self) -> usize {
		if self.strings.is_empty() {
            0
        } else {
            self.strings.len() - 1
        }
	}
}