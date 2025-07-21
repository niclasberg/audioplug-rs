use std::{
    any::Any,
    cell::Cell,
    ops::{Range, RangeInclusive},
};

use super::{
    AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo,
    ParameterTraversal, ParseError, PlainValue,
};

pub struct IntParameter {
    info: IntParameterInfo,
    value: Cell<i64>,
}

impl IntParameter {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        let info = IntParameterInfo::new(id, name);
        let value = Cell::new(info.default);
        Self { info, value }
    }

    pub fn with_range(mut self, range: impl Into<IntRange>) -> Self {
        self.info.range = range.into();
        self
    }
}

impl AnyParameter for IntParameter {
    fn info(&self) -> &dyn ParameterInfo {
        &self.info
    }

    fn plain_value(&self) -> PlainValue {
        PlainValue::new(self.value.get() as _)
    }

    fn set_value_normalized(&self, value: NormalizedValue) {
        self.set_value_plain(self.info.denormalize(value));
    }

    fn set_value_plain(&self, value: PlainValue) {
        self.value.replace(value.0.round() as _);
    }

    fn as_param_ref(&self) -> ParamRef {
        ParamRef::Int(self)
    }
}

impl Parameter<i64> for IntParameter {
    fn value(&self) -> i64 {
        self.value.get()
    }

    fn set_value(&self, value: i64) {
        self.value.replace(value);
    }
}

impl ParameterTraversal for IntParameter {
    fn visit<V: super::ParamVisitor>(&self, visitor: &mut V) {
        visitor.int_parameter(self);
    }
}

pub struct IntParameterInfo {
    id: ParameterId,
    name: &'static str,
    range: IntRange,
    default: i64,
}

impl IntParameterInfo {
    fn new(id: ParameterId, name: &'static str) -> Self {
        Self {
            id,
            name,
            range: IntRange::Linear { min: 0, max: 1 },
            default: 0,
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
        self.name
    }

    fn default_value(&self) -> PlainValue {
        PlainValue::new(self.default as f64)
    }

    fn min_value(&self) -> PlainValue {
        self.range.min_value()
    }

    fn max_value(&self) -> PlainValue {
        self.range.max_value()
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

    fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError> {
        let plain_value = str.parse().map_err(|_| ParseError)?;
        Ok(self.normalize(PlainValue::new(plain_value)))
    }

    fn string_from_value(&self, value: NormalizedValue) -> String {
        let plain_value = self.denormalize(value);
        plain_value.0.round().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntRange {
    Linear { min: i64, max: i64 },
}

impl From<Range<i64>> for IntRange {
    fn from(value: Range<i64>) -> Self {
        Self::Linear {
            min: value.start,
            max: value.end + 1,
        }
    }
}

impl From<RangeInclusive<i64>> for IntRange {
    fn from(value: RangeInclusive<i64>) -> Self {
        Self::Linear {
            min: *value.start(),
            max: *value.end(),
        }
    }
}

impl IntRange {
    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
        let value = match self {
            IntRange::Linear { min, max } => (value.0 - *min as f64) / (*max - *min) as f64,
        };
        NormalizedValue(value)
    }

    pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        let value = match self {
            IntRange::Linear { min, max } => *min as f64 + value.0 * (*max - *min) as f64,
        };
        PlainValue(value)
    }

    pub fn min_value(&self) -> PlainValue {
        let value = match self {
            Self::Linear { min, .. } => *min,
        };
        PlainValue(value as _)
    }

    pub fn max_value(&self) -> PlainValue {
        let value = match self {
            Self::Linear { max, .. } => *max,
        };
        PlainValue(value as _)
    }

    pub fn steps(&self) -> usize {
        match self {
            IntRange::Linear { min, max } => (max - min).unsigned_abs() as usize,
        }
    }
}
