use super::{NormalizedValue, PlainValue};

pub struct FloatParameter {
    name: &'static str,
    range: FloatRange,
    default: f64
}

impl FloatParameter {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            range: FloatRange::Linear { min: 0.0, max: 1.0 },
            default: 0.0
        }
    }

    pub fn with_range(mut self, range: FloatRange) -> Self {
        self.range = range;
        self
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
        let Self::Linear { min, max } = self;
		NormalizedValue((value.0 - min.clamp(*min, *max)) / max)
    }
}