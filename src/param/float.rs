use std::cell::Cell;

use super::{NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

pub struct FloatParameter {
    info: FloatParameterInfo,
    value: Cell<f64>,
}

impl FloatParameter {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        let info = FloatParameterInfo::new(id, name);
        let value = info.default.into();
        Self {
            info,
            value
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.internal_set_value(value);
    }

    pub(crate) fn internal_set_value(&self, value: f64) {
        self.value.replace(value);
    }

    pub fn with_range(mut self, range: FloatRange) -> Self {
        self.info.range = range;
        self
    }

    pub fn with_default(mut self, default_value: f64) -> Self {
        self.info.default = default_value;
        self.value.set(default_value);
        self
    }
}

impl Parameter<f64> for FloatParameter {
	fn info(&self) -> &dyn ParameterInfo {
		&self.info
	}

	fn value(&self) -> f64 {
        self.value.get()
    }

	fn plain_value(&self) -> PlainValue {
        PlainValue(self.value())
    }

	fn set_value(&self, value: f64) {
		self.value.replace(value);
	}

	fn set_value_normalized(&self, value: NormalizedValue) {
		self.value.replace(self.info.denormalize(value).0);
	}

	fn as_param_ref(&self) -> ParamRef {
        ParamRef::Float(self)
    }
}

pub struct FloatParameterInfo {
	id: ParameterId,
    name: &'static str,
    range: FloatRange,
    default: f64
}

impl FloatParameterInfo {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        Self {
			id, 
            name,
            range: FloatRange::Linear { min: 0.0, max: 1.0 },
            default: 0.0
        }
    }

    pub fn range(&self) -> FloatRange {
        self.range
    }
}

impl ParameterInfo for FloatParameterInfo {
	fn id(&self) -> super::ParameterId {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn default_value(&self) -> PlainValue {
		PlainValue::new(self.default)
	}
	
	fn normalize(&self, value: PlainValue) -> NormalizedValue {
		self.range.normalize(value)
	}
	
	fn denormalize(&self, value: NormalizedValue) -> PlainValue {
		self.range.denormalize(value)
	}
	
	fn step_count(&self) -> usize {
		0
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatRange {
    Linear { min: f64, max: f64 }
}

impl FloatRange {
    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
		let value = match self {
			Self::Linear { min, max } => (value.0 - *min) / (*max - *min)
		};
		NormalizedValue(value)
    }

	pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
		let value = match self {
			Self::Linear { min, max } => *min + value.0 * (*max - *min)
		};
		PlainValue(value)
	}
}