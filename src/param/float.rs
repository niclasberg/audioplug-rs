use std::{any::Any, cell::Cell};

use super::{
    AnyParameter, NormalizedValue, ParamRef, ParamVisitor, Parameter, ParameterId, ParameterInfo,
    ParameterTraversal, ParseError, PlainValue,
};

pub struct FloatParameter {
    info: FloatParameterInfo,
    value: Cell<f64>,
}

impl FloatParameter {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        let info = FloatParameterInfo::new(id, name);
        let value = info.default.into();
        Self { info, value }
    }

    pub fn set_value(&mut self, value: f64) {
        self.internal_set_value(value);
    }

    pub(crate) fn internal_set_value(&self, value: f64) {
        self.value.replace(value);
    }

    pub fn with_range(mut self, range: impl Into<FloatRange>) -> Self {
        self.info.range = range.into();
        *self.value.get_mut() = self
            .value
            .get()
            .clamp(self.info.min_value().0, self.info().max_value().0);
        self
    }

    pub fn with_linear_range(mut self, min: f64, max: f64) -> Self {
        self.info.range = FloatRange::Linear { min, max };
        *self.value.get_mut() = self
            .value
            .get()
            .clamp(self.info.min_value().0, self.info().max_value().0);
        self
    }

    pub fn with_default(mut self, default_value: f64) -> Self {
        self.info.default = default_value;
        self.value.set(default_value);
        self
    }
}

impl AnyParameter for FloatParameter {
    fn info(&self) -> &dyn ParameterInfo {
        &self.info
    }

    fn plain_value(&self) -> PlainValue {
        PlainValue(self.value())
    }

    fn set_value_normalized(&self, value: NormalizedValue) {
        self.value.replace(self.info.denormalize(value).0);
    }

    fn as_param_ref(&self) -> ParamRef {
        ParamRef::Float(self)
    }
}

impl Parameter<f64> for FloatParameter {
    fn value(&self) -> f64 {
        self.value.get()
    }

    fn set_value(&self, value: f64) {
        self.value.replace(value);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ParameterTraversal for FloatParameter {
    fn visit<V: ParamVisitor>(&self, visitor: &mut V) {
        visitor.float_parameter(self)
    }
}

pub struct FloatParameterInfo {
    id: ParameterId,
    name: &'static str,
    range: FloatRange,
    default: f64,
}

impl FloatParameterInfo {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        Self {
            id,
            name,
            range: FloatRange::Linear { min: 0.0, max: 1.0 },
            default: 0.0,
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
        self.name
    }

    fn default_value(&self) -> PlainValue {
        PlainValue::new(self.default)
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
        0
    }

    fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError> {
        let value = str.parse().map_err(|_| ParseError)?;
        NormalizedValue::from_f64(value).ok_or(ParseError)
    }

    fn string_from_value(&self, value: NormalizedValue) -> String {
        value.0.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatRange {
    Linear { min: f64, max: f64 },
}

impl FloatRange {
    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
        let value = match self {
            Self::Linear { min, max } => (value.0 - *min) / (*max - *min),
        };
        NormalizedValue(value)
    }

    pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        let value = match self {
            Self::Linear { min, max } => *min + value.0 * (*max - *min),
        };
        PlainValue(value)
    }

    pub fn min_value(&self) -> PlainValue {
        let value = match self {
            Self::Linear { min, .. } => *min,
        };
        PlainValue(value)
    }

    pub fn max_value(&self) -> PlainValue {
        let value = match self {
            Self::Linear { max, .. } => *max,
        };
        PlainValue(value)
    }
}
