use std::fmt::Display;

mod bool;
mod float;
mod int;
mod string_list;

use bool::BoolParameter;
pub use float::{FloatParameter, FloatParameterInfo, FloatRange};
pub use int::{IntParameter, IntParameterInfo, IntRange};
pub use string_list::StringListParameter;

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

/// Normalized parameter value, in range 0.0 to 1.0
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct NormalizedValue(pub(super) f64);

impl NormalizedValue {
    #[inline]
    pub unsafe fn from_f64_unchecked(value: f64) -> Self {
        Self(value)
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

/// Plain parameter value
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct PlainValue(f64);

impl PlainValue {
    #[inline]
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}

impl Into<f64> for PlainValue {
    fn into(self) -> f64 {
        self.0
    }
}

pub trait Parameter<T> {
    type Info: ParameterInfo;
    fn info(&self) -> &Self::Info;
    fn plain_value(&self) -> PlainValue;
    fn normalized_value(&self) -> NormalizedValue {
        self.info().normalize(self.plain_value())
    }
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
}

impl<T: Parameter> ParameterInfo for T {
    fn id(&self) -> ParameterId {
        self.info().id()
    }

    fn name(&self) -> &str {
        self.info().name()
    }

    fn default_value(&self) -> PlainValue {
        self.info().default_value()
    }
	
	fn normalize(&self, value: PlainValue) -> NormalizedValue {
		self.info().normalize(value)
	}
	
	fn denormalize(&self, value: NormalizedValue) -> PlainValue {
		self.info().denormalize(value)
	}
	
	fn step_count(&self) -> usize {
		self.info().step_count()
	}
}

pub enum ParamRef<'a> {
    Float(&'a FloatParameter),
    Int(&'a IntParameter),
    StringList(&'a StringListParameter),
    ByPass,
    Bool(&'a BoolParameter)
}

impl<'a> ParamRef<'a> {
    pub fn info(&self) -> &dyn ParameterInfo {
        match self {
            Self::ByPass => todo!(),
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
            Self::Float(p) => p.plain_value(),
            Self::Int(p) => p.plain_value(),
            Self::StringList(_) => p.plain_value(),
            Self::ByPass => p.plain_value(),
            Self::Bool(_) => p.plain_value(),
        }
    }

    pub fn get_plain(&self) -> PlainValue {
        match self {
            Self::Float(p) => p.plain_value(),
            Self::Int(p) => p.plain_value(),
            Self::StringList(_) => p.plain_value(),
            Self::ByPass => p.plain_value(),
            Self::Bool(_) => p.plain_value(),
        }
    }

    pub fn get_normalized(&self) -> NormalizedValue {
        match self {
            Self::Float(p) => p.info().range().normalize(p.plain_value()),
            Self::Int(_) => todo!(),
            Self::StringList(_) => todo!(),
            Self::ByPass => todo!(),
            Self::Bool(_) => todo!(),
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

pub type ParameterGetter<P: Params> = fn(&P) -> ParamRef;

pub trait Params: Default + 'static {
    const PARAMS: &'static [ParameterGetter<Self>];
}

impl Params for () {
    const PARAMS: &'static [ParameterGetter<Self>] = &[];
}
