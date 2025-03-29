use std::{any::Any, fmt::Display};

mod bool;
mod bypass;
mod float;
mod group;
mod int;
mod param_ref;
mod parameter_map;
mod string_list;
mod traversal;

pub use bool::{BoolParameter, BoolParameterInfo};
pub use bypass::ByPassParameter;
pub use float::{FloatParameter, FloatParameterInfo, FloatRange};
pub use group::{AnyParameterGroup, ParameterGroup};
pub use int::{IntParameter, IntParameterInfo, IntRange};
pub use param_ref::ParamRef;
pub use parameter_map::{AnyParameterMap, ParameterMap, Params};
pub use string_list::StringListParameter;
pub use traversal::{ParamVisitor, ParameterTraversal};

use crate::app::ParamSignal;

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
        if value >= 0.0 && value <= 1.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn from_f64_unchecked(value: f64) -> Self {
        Self(value)
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

impl Into<f64> for NormalizedValue {
    fn into(self) -> f64 {
        self.0
    }
}

impl Into<bool> for NormalizedValue {
    fn into(self) -> bool {
        self.0 > 0.5
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

impl Into<f64> for PlainValue {
    fn into(self) -> f64 {
        self.0
    }
}

pub trait AnyParameter: Any {
    fn info(&self) -> &dyn ParameterInfo;
    fn plain_value(&self) -> PlainValue;
    fn normalized_value(&self) -> NormalizedValue {
        self.info().normalize(self.plain_value())
    }
    fn set_value_normalized(&self, value: NormalizedValue);
    fn set_value_plain(&self, value: PlainValue) {
        self.set_value_normalized(self.info().normalize(value));
    }
    fn as_signal_plain(&self) -> ParamSignal<PlainValue>
    where
        Self: Sized,
    {
        ParamSignal::new_plain(self)
    }
    fn as_signal_normalized(&self) -> ParamSignal<NormalizedValue>
    where
        Self: Sized,
    {
        ParamSignal::new_normalized(self)
    }
    fn as_param_ref(&self) -> ParamRef;
}

pub trait Parameter<T>: AnyParameter {
    fn value(&self) -> T;
    fn set_value(&self, value: T);

    fn as_any(&self) -> &dyn Any;
    fn as_signal(&self) -> ParamSignal<T>
    where
        Self: Sized,
    {
        ParamSignal::new(self)
    }
}

pub trait ParameterInfo {
    fn id(&self) -> ParameterId;
    fn name(&self) -> &str;
    fn default_value(&self) -> PlainValue;
    fn min_value(&self) -> PlainValue;
    fn max_value(&self) -> PlainValue;
    fn normalize(&self, value: PlainValue) -> NormalizedValue;
    fn denormalize(&self, value: NormalizedValue) -> PlainValue;
    fn step_count(&self) -> usize;
    fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError>;
    fn string_from_value(&self, value: NormalizedValue) -> String;
}
