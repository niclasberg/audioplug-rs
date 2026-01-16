use std::cell::Cell;

use crate::{param::Parameter, ui::ReadSignal};

use super::{
    AnyParameter, NormalizedValue, ParamRef, ParamVisitor, ParameterId, ParameterTraversal,
    ParseError, PlainValue,
};

pub struct FloatParameter {
    id: ParameterId,
    name: &'static str,
    range: FloatRange,
    default: f64,
    value: Cell<f64>,
}

impl FloatParameter {
    pub fn new(id: ParameterId, name: &'static str) -> Self {
        Self {
            id,
            name,
            range: FloatRange::Linear { min: 0.0, max: 1.0 },
            default: 0.0,
            value: Cell::new(0.0),
        }
    }

    pub(crate) fn set_value(&self, value: f64) {
        self.value.replace(value);
    }

    pub fn with_range(mut self, range: impl Into<FloatRange>) -> Self {
        self.range = range.into();
        *self.value.get_mut() = self
            .value
            .get()
            .clamp(self.min_value().0, self.max_value().0);
        self
    }

    pub fn with_linear_range(mut self, min: f64, max: f64) -> Self {
        self.range = FloatRange::Linear { min, max };
        *self.value.get_mut() = self
            .value
            .get()
            .clamp(self.min_value().0, self.max_value().0);
        self
    }

    pub fn with_default(mut self, default_value: f64) -> Self {
        self.default = default_value;
        self.value.set(default_value);
        self
    }

    pub fn range(&self) -> FloatRange {
        self.range
    }

    pub fn value(&self) -> f64 {
        self.value.get()
    }

    pub fn as_signal(&self) -> ReadSignal<f64> {
        ReadSignal::from_parameter(self.id, |param_ref| match param_ref {
            ParamRef::Float(p) => p.value(),
            _ => unreachable!(),
        })
    }
}

impl super::private::Sealed for FloatParameter {}

impl AnyParameter for FloatParameter {
    fn id(&self) -> super::ParameterId {
        self.id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn default_value_plain(&self) -> PlainValue {
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

impl Parameter for FloatParameter {
    type Value = f64;

    fn default_value(&self) -> Self::Value {
        self.default
    }

    fn plain_value(&self, value: f64) -> PlainValue {
        PlainValue(value)
    }

    fn value_from_plain(&self, value: PlainValue) -> Self::Value {
        value.0
    }

    fn value_from_normalized(&self, value: NormalizedValue) -> Self::Value {
        self.denormalize(value).0
    }

    fn downcast_param_ref<'s>(param_ref: ParamRef<'s>) -> Option<&'s Self> {
        match param_ref {
            ParamRef::Float(p) => Some(p),
            _ => None,
        }
    }
}

impl ParameterTraversal for FloatParameter {
    fn visit<V: ParamVisitor>(&self, visitor: &mut V) {
        visitor.float_parameter(self)
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
