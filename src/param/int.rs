use std::{any::Any, cell::Cell};

use super::{AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

pub struct IntParameter {
    info: IntParameterInfo,
    value: Cell<i64>
}

impl IntParameter {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        let info = IntParameterInfo::new(id, name);
        let value = Cell::new(info.default);
        Self {
            info,
            value
        }
    }

    pub fn with_range(mut self, range: IntRange) -> Self {
        self.info.range = range;
        self
    }
}

impl AnyParameter for IntParameter {
	fn info(&self) -> &dyn ParameterInfo {
		&self.info 
	}

	fn plain_value(&self) -> PlainValue {
		todo!()
	}

	fn set_value_normalized(&self, value: NormalizedValue) {
		todo!()
	}
}

impl Parameter for IntParameter {
	type Value = i64;

	fn value(&self) -> i64 {
        self.value.get()
    }
	
	fn set_value(&self, value: i64) {
        self.value.replace(value);
    }
	
	fn as_param_ref(&self) -> ParamRef {
        ParamRef::Int(self)
    }

	fn as_any(&self) -> &dyn Any {
		self
	}
}

pub struct IntParameterInfo {
	id: ParameterId,
    name: &'static str,
    range: IntRange,
    default: i64
}

impl IntParameterInfo {
    fn new(id: ParameterId, name: &'static str) -> Self {
        Self { 
			id,
            name, 
            range: IntRange::Linear { min: 0, max: 1 }, 
            default: 0
        }
    }

    pub fn range(&self) -> IntRange {
        self.range
    }
}

impl ParameterInfo for IntParameterInfo {
	fn id(&self) -> ParameterId {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn default_value(&self) -> PlainValue {
		PlainValue::new(self.default as f64)
	}
	
	fn normalize(&self, value: PlainValue) -> NormalizedValue {
		self.range.normalize(value)
	}
	
	fn denormalize(&self, value: NormalizedValue) -> PlainValue {
		self.range.denormalize(value)
	}
	
	fn step_count(&self) -> usize {
		self.range.steps()
	}
	
	fn value_from_string(&self, str: &str) -> Result<NormalizedValue, super::ParseError> {
		todo!()
	}

	fn string_from_value(&self, value: NormalizedValue) -> String {
		todo!()
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntRange {
    Linear { min: i64, max: i64 }
}

impl IntRange {
    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
		let value = match self {
			IntRange::Linear { min, max } => 
				(value.0 - *min as f64) / (*max - *min) as f64
		};
		NormalizedValue(value)
    }

	pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
		let value = match self {
			IntRange::Linear { min, max } => 
				*min as f64 + value.0 * (*max - *min) as f64,
		};
        PlainValue(value)
	}

    pub fn steps(&self) -> usize {
        match self {
            IntRange::Linear { min, max } => (max - min + 1) as usize,
        }
    }
}