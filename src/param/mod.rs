use std::{any::Any, fmt::Display};

mod bool;
mod float;
mod int;
mod string_list;
mod bypass;
mod macros;
mod parameter_map;

pub use bool::{BoolParameter, BoolParameterInfo};
pub use bypass::ByPassParameter;
pub use float::{FloatParameter, FloatParameterInfo, FloatRange};
pub use int::{IntParameter, IntParameterInfo, IntRange};
pub use string_list::StringListParameter;
pub use parameter_map::{ParameterMap, AnyParameterMap, Params, ParameterGetter};

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
    Custom(&'static str)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, Hash)]
pub struct ParameterId(u32);

impl ParameterId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

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
pub struct PlainValue(f64);

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

pub trait ParameterBase {
	
}

pub trait Parameter<T> {
    fn info(&self) -> &dyn ParameterInfo;
	fn value(&self) -> T;
    fn plain_value(&self) -> PlainValue;
	fn normalized_value(&self) -> NormalizedValue {
		self.info().normalize(self.plain_value())
	}

	fn set_value(&self, value: T);
    fn set_value_normalized(&self, value: NormalizedValue);
	
	fn as_param_ref(&self) -> ParamRef;
}

pub trait ParameterInfo {
    fn id(&self) -> ParameterId;
    fn name(&self) -> &str;
    fn default_value(&self) -> PlainValue;
	fn normalize(&self, value: PlainValue) -> NormalizedValue;
	fn denormalize(&self, value: NormalizedValue) -> PlainValue;
	fn step_count(&self) -> usize;
    fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError>;
    fn string_from_value(&self, value: NormalizedValue) -> String;
}

pub enum ParamRef<'a> {
    Float(&'a FloatParameter),
    Int(&'a IntParameter),
    StringList(&'a StringListParameter),
    ByPass(&'a ByPassParameter),
    Bool(&'a BoolParameter),
}

impl<'a> ParamRef<'a> {
    pub fn info(&self) -> &dyn ParameterInfo {
        match self {
            Self::ByPass(p) => p.info(),
            Self::Float(p) => p.info(),
            Self::Int(p) => p.info(),
            Self::StringList(p) => p.info(),
            Self::Bool(p) => p.info(),
        }
    }

    pub fn name(&self) -> &str {
        self.info().name()
    }

    pub fn id(&self) -> ParameterId {
        self.info().id()
    }

    pub fn default_value(&self) -> PlainValue {
        self.info().default_value()
    }

    pub fn default_normalized(&self) -> NormalizedValue {
        self.normalize(self.default_value())
    }

    pub(crate) fn internal_set_value_normalized(&self, value: NormalizedValue) {
        match self {
            Self::Float(p) => p.set_value_normalized(value),
            Self::Int(p) => p.set_value_normalized(value),
            Self::StringList(p) => p.set_value_normalized(value),
            Self::ByPass(p) => p.set_value_normalized(value),
            Self::Bool(p) => p.set_value_normalized(value),
        }
    }

	pub(crate) fn internal_set_value_plain(&self, value: PlainValue) {
		self.internal_set_value_normalized(self.normalize(value))
	}

    pub fn plain_value(&self) -> PlainValue {
        match self {
            Self::Float(p) => p.plain_value(),
            Self::Int(p) => p.plain_value(),
            Self::StringList(p) => p.plain_value(),
            Self::ByPass(p) => p.plain_value(),
            Self::Bool(p) => p.plain_value(),
        }
    }

    pub fn normalized_value(&self) -> NormalizedValue {
        match self {
            Self::Float(p) => p.normalized_value(),
            Self::Int(p) => p.normalized_value(),
            Self::StringList(p) => p.normalized_value(),
            Self::ByPass(p) => p.normalized_value(),
            Self::Bool(p) => p.normalized_value(),
        }
    }

    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
        self.info().normalize(value)
    }

    pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        self.info().denormalize(value)
    }

    pub fn step_count(&self) -> usize {
        self.info().step_count()
    }
}

