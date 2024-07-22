use super::{NormalizedValue, ParamRef, PlainValue};

pub struct IntParameter {
    info: IntParameterInfo,
    value: i64
}

impl IntParameter {
    pub fn new(name: &'static str) -> Self {
        let info = IntParameterInfo::new(name);
        let value = info.default;
        Self {
            info,
            value
        }
    }

    pub fn info(&self) -> &IntParameterInfo {
        &self.info
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn set_value(&mut self, value: i64) {
        self.value = value;
    }

    pub fn with_range(mut self, range: IntRange) -> Self {
        self.info.range = range;
        self
    }

    pub fn as_param_ref(&mut self) -> ParamRef {
        ParamRef::Int(self)
    }
}

pub struct IntParameterInfo {
    name: &'static str,
    range: IntRange,
    default: i64
}

impl IntParameterInfo {
    fn new(name: &'static str) -> Self {
        Self { 
            name, 
            range: IntRange::Linear { min: 0, max: 1 }, 
            default: 0
        }
    }

    pub fn range(&self) -> IntRange {
        self.range
    }

	pub fn name(&self) -> &'static str {
		&self.name
	}

	pub fn default_value(&self) -> PlainValue {
		PlainValue::new(self.default as f64)
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