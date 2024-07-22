use std::cell::Cell;

use super::{NormalizedValue, ParamRef, PlainValue};

pub struct FloatParameter {
    info: FloatParameterInfo,
    value: Cell<f64>,
}

impl FloatParameter {
    pub fn new(name: &'static str) -> Self {
        let info = FloatParameterInfo::new(name);
        let value = info.default.into();
        Self {
            info,
            value
        }
    }

    pub fn info(&self) -> &FloatParameterInfo {
        &self.info
    }

    pub fn value(&self) -> f64 {
        self.value.get()
    }

    pub fn plain_value(&self) -> PlainValue {
        PlainValue(self.value())
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

    pub fn as_param_ref(&mut self) -> ParamRef {
        ParamRef::Float(self)
    }
}

pub struct FloatParameterInfo {
    name: &'static str,
    range: FloatRange,
    default: f64
}

impl FloatParameterInfo {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            range: FloatRange::Linear { min: 0.0, max: 1.0 },
            default: 0.0
        }
    }

    pub fn range(&self) -> FloatRange {
        self.range
    }

	pub fn name(&self) -> &'static str {
		&self.name
	}

	pub fn default_value(&self) -> PlainValue {
		PlainValue::new(self.default)
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