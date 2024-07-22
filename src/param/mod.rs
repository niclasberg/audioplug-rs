use std::fmt::Display;

mod bool;
mod float;
mod int;
mod string_list;

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

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
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

pub trait Parameter {
    type Info: ParameterInfo;
    fn info(&self) -> &Self::Info;
    fn get_plain_value(&self) -> PlainValue;
    fn get_normalized_value(&self) -> NormalizedValue;
}

pub trait ParameterInfo {
    fn id(&self) -> ParameterId;
    fn name(&self) -> &str;
    fn default_value(&self) -> PlainValue;
}

impl ParameterInfo for Parameter {
    fn id(&self) -> ParameterId {
        self.info().id()
    }

    fn name(&self) -> &str {
        self.info().name()
    }

    fn default_value(&self) -> PlainValue {
        self.info().default_value()
    }
}

pub enum ParamRef<'a> {
    Float(&'a mut FloatParameter),
    Int(&'a mut IntParameter),
    StringList(&'a mut StringListParameter),
    ByPass,
    Bool(&'a mut BoolParameter)
}

impl<'a> ParamRef<'a> {
    pub fn info(&self) -> &dyn ParameterInfo {
        match self {
            Self::ByPass => ParameterId(0),
            Self::Float(p) => p.info().name(),
            Self::Int(p) => p.info().name(),
            Self::StringList(p) => p.name(),
            Self::Bool(p) => p.name,
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
        self.normalize(self.default())
    }

    pub fn get_plain(&self) -> PlainValue {
        match self {
            Self::Float(p) => p.plain_value(),
            Self::Int(_) => todo!(),
            Self::StringList(_) => todo!(),
            Self::ByPass => todo!(),
            Self::Bool(_) => todo!(),
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
        match self {
            Self::Float(p) => p.info().range().normalize(value),
            Self::Int(p) => p.info().range().normalize(value),
            Self::StringList(p) => NormalizedValue(value.0.clamp(0.0, p.string_count() as f64) / (p.string_count() as f64)),
            Self::ByPass | Self::Bool(_) => NormalizedValue(value.0),
        }
    }

    pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        match self {
            Self::Float(p) => p.info().range().denormalize(value),
            Self::Int(p) => p.info().range().denormalize(value),
            Self::StringList(p) => PlainValue(value.0 * (p.string_count() as f64)),
            Self::ByPass | Self::Bool(_) => PlainValue(value.0),
        }
    }

    pub fn step_count(&self) -> usize {
        match self {
            Self::ByPass | Self::Bool(_) => 1,
            Self::Int(p) => p.info().range().steps(),
            Self::Float(FloatParameter {.. }) => 0,
            Self::StringList(string_list) => string_list.step_count()
        }
    }
}

type ParameterGetter<P: Params> = fn(&mut P) -> ParamRef;

pub trait Params: Default + 'static {
    const PARAMS: &'static [ParameterGetter<Self>];
}

impl Params for () {
    const PARAMS: &'static [fn(&Self) -> ParamRef] = &[];
}
