use std::{any::Any, fmt::Display};

mod bool;
mod bypass;
mod float;
mod group;
mod int;
mod parameter_map;
mod string_list;
mod traversal;

pub use bool::BoolParameter;
pub use bypass::ByPassParameter;
pub use float::{FloatParameter, FloatRange};
pub use group::{AnyParameterGroup, ParameterGroup};
pub use int::{IntParameter, IntRange};
pub use parameter_map::{AnyParameterMap, ParamRef, ParameterMap, Params};
pub use string_list::StringListParameter;
pub use traversal::{ParamVisitor, ParameterTraversal};

use crate::ui::ReadSignal;

#[derive(Clone, Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to parse parameter value from string")
    }
}

pub enum Unit {
    None,
    Decibels,
    MiliSeconds,
    Seconds,
    Custom(&'static str),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, Hash)]
pub struct ParameterId(pub u32);

impl From<ParameterId> for u32 {
    fn from(value: ParameterId) -> Self {
        value.0
    }
}

impl From<ParameterId> for u64 {
    fn from(value: ParameterId) -> Self {
        value.0.into()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, Hash)]
pub struct GroupId(pub u32);

impl From<GroupId> for i32 {
    fn from(value: GroupId) -> Self {
        value.0 as _
    }
}

impl From<GroupId> for u32 {
    fn from(value: GroupId) -> Self {
        value.0
    }
}

impl From<GroupId> for u64 {
    fn from(value: GroupId) -> Self {
        value.0.into()
    }
}

/// Normalized parameter value, in range 0.0 to 1.0
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct NormalizedValue(pub(super) f64);

impl NormalizedValue {
    pub fn from_f64(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    #[inline]
    /// Caller must ensure that the value is between 0.0 and 1.0
    pub(crate) fn from_f64_unchecked(value: f64) -> Self {
        Self(value)
    }

    pub fn from_i64(value: i64) -> Self {
        Self(value as f64)
    }

    #[inline]
    fn from_bool(value: bool) -> Self {
        Self(if value { 1.0 } else { 0.0 })
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<NormalizedValue> for f64 {
    fn from(val: NormalizedValue) -> Self {
        val.0
    }
}

impl From<NormalizedValue> for bool {
    fn from(val: NormalizedValue) -> Self {
        val.0 > 0.5
    }
}

/// Plain parameter value
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct PlainValue(pub f64);

impl PlainValue {
    #[inline]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    #[inline]
    fn from_bool(value: bool) -> Self {
        Self(if value { 1.0 } else { 0.0 })
    }
}

impl From<PlainValue> for f64 {
    fn from(val: PlainValue) -> Self {
        val.0
    }
}

pub(super) mod private {
    pub trait Sealed {}
}

pub trait AnyParameter: Any + private::Sealed {
    fn id(&self) -> ParameterId;
    fn name(&self) -> &str;
    fn default_value_plain(&self) -> PlainValue;
    fn min_value(&self) -> PlainValue;
    fn max_value(&self) -> PlainValue;
    fn normalize(&self, value: PlainValue) -> NormalizedValue;
    fn denormalize(&self, value: NormalizedValue) -> PlainValue;
    fn step_count(&self) -> usize;
    fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError>;
    fn string_from_value(&self, value: NormalizedValue) -> String;

    fn set_value_normalized(&self, value: NormalizedValue);
    fn set_value_plain(&self, value: PlainValue) {
        self.set_value_normalized(self.normalize(value));
    }
    fn as_signal_plain(&self) -> ReadSignal<PlainValue>
    where
        Self: Sized,
    {
        ReadSignal::from_parameter(self.id())
    }
    fn as_signal_normalized(&self) -> ReadSignal<NormalizedValue>
    where
        Self: Sized,
    {
        ReadSignal::from_parameter(self.id())
    }
}

pub trait Parameter: AnyParameter {
    type Value: Any;
    fn default_value(&self) -> Self::Value;
    fn plain_value(&self, value: Self::Value) -> PlainValue;
    fn normalized_value(&self, value: Self::Value) -> NormalizedValue {
        self.normalize(self.plain_value(value))
    }
}
