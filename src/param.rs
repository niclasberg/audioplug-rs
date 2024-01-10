use std::{fmt::Display, marker::PhantomData, ops::Index};

#[derive(Clone, Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to parse parameter value from string")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntRange {
    Linear { min: i64, max: i64 }
}

impl IntRange {
    pub fn normalize(&self, value: f64) -> f64 {
        match self {
            IntRange::Linear { min, max } => (value - *min as f64) / (*max as f64),
        }
    }

    pub fn steps(&self) -> usize {
        match self {
            IntRange::Linear { min, max } => (max - min + 1) as usize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatRange {
    Linear { min: f64, max: f64 }
}

impl FloatRange {
    pub fn normalize(&self, value: f64) -> f64 {
        match self {
            Self::Linear { min, max } => (value - min.clamp(*min, *max)) / max
        }
    }
}

pub enum Unit {
    None,
    Decibels,
    MiliSeconds,
    Seconds,
    Custom(&'static str)
}

/// Normalized parameter value, in range 0.0 to 1.0
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct NormalizedValue(f64);

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

pub enum ParameterValue {
    Float(f64),
    Int(i64),
    StringList(i64),
    ByPass(bool),
}

pub enum Parameter {
    Float(FloatParameter),
    Int(IntParameter),
    StringList(StringListParameter),
    ByPass,
    Bool(BoolParameter)
}

impl Parameter {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ByPass => "Bypass",
            Self::Float(p) => p.name,
            Self::Int(p) => p.name,
            Self::StringList(StringListParameter{ name, .. }) => name,
            Self::Bool(p) => p.name,
        }
    }

    pub fn default(&self) -> PlainValue {
        let value = match self {
            Parameter::Float(p) => p.default,
            Parameter::Int(p) => p.default as f64,
            Parameter::StringList(p) => p.default_index as f64,
            Parameter::ByPass => 0.0,
            Parameter::Bool(p) => if p.default { 1.0 } else { 0.0 }
        };
        PlainValue(value)
    }

    pub fn default_normalized(&self) -> NormalizedValue {
        self.normalize(self.default())
    }

    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
        let value = value.0;
        let normalized_value = match self {
            Parameter::Float(p) => p.range.normalize(value),
            Parameter::Int(p) => p.range.normalize(value),
            Parameter::StringList(p) => value.clamp(0.0, p.string_count() as f64) / (p.string_count() as f64),
            Parameter::ByPass | Parameter::Bool(_) => value,
        };
        NormalizedValue(normalized_value)  
    }

    pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        let value = value.0;
        let plain_value = match self {
            Parameter::Float(p) => todo!(),
            Parameter::Int(_) => todo!(),
            Parameter::StringList(p) => value * (p.string_count() as f64),
            Parameter::ByPass | Parameter::Bool(_) => value,
        };
        PlainValue(plain_value)
    }

    pub fn step_count(&self) -> usize {
        match self {
            Self::ByPass | Self::Bool(_) => 1,
            Self::Int(p) => p.range.steps(),
            Self::Float(FloatParameter {.. }) => 0,
            Self::StringList(string_list) => string_list.step_count()
        }
    }
}

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
}

impl From<FloatParameter> for Parameter {
    fn from(value: FloatParameter) -> Self {
        Parameter::Float(value)
    }
}

pub struct IntParameter {
    name: &'static str,
    range: IntRange,
    default: i64
}

impl IntParameter {
    pub fn new(name: &'static str) -> Self {
        Self { 
            name, 
            range: IntRange::Linear { min: 0, max: 1 }, 
            default: 0
        }
    }

    pub fn range(&self) -> IntRange {
        self.range
    }
}

pub struct StringListParameter {
    name: &'static str,
    strings: Vec<String>,
    default_index: usize,
}

impl StringListParameter {
    pub fn new(name: &'static str, strings: impl Into<Vec<String>>, default_index: usize) -> Self {
        Self {
            name, 
            strings: strings.into(),
            default_index
        }
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn step_count(&self) -> usize {
        if self.strings.is_empty() {
            0
        } else {
            self.strings.len() - 1
        }
    }

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.strings.iter().position(|x| x == key)
    }
}

pub struct BoolParameter {
    name: &'static str,
    default: bool
}

impl BoolParameter {
    pub fn new(name: &'static str, default: bool) -> Self {
        Self { name, default }
    }
}

pub trait ParameterSet<Store> {
    fn init_store(&self) -> Store;
    fn parameter_ref(&self, index: usize) -> &Parameter;
    fn set_normalized(&self, store: &mut Store, index: usize, value: NormalizedValue);
    fn get_normalized(&self, store: &Store, index: usize) -> NormalizedValue;
}

impl ParameterSet<Vec<PlainValue>> for Vec<Parameter> {
    fn init_store(&self) -> Vec<PlainValue> {
        let mut store = Vec::with_capacity(self.len());
        for param in self.iter() {
            store.push(param.default());
        }
        store
    }

    fn set_normalized(&self, store: &mut Vec<PlainValue>, index: usize, value: NormalizedValue) {
        debug_assert!(index < self.len());

        if let Some(param) = self.get(index) {
            store[index] = param.denormalize(value);
        }
    }

    fn get_normalized(&self, store: &Vec<PlainValue>, index: usize) -> NormalizedValue {
        debug_assert!(index < self.len());
        
        if let Some(param) = self.get(index) {
            param.normalize(store[index])
        } else {
            NormalizedValue(0.0)
        }
    }

    fn parameter_ref(&self, index: usize) -> &Parameter {
        &self[index]
    }
}


pub struct ParameterRef<S, FGet, FSet> 
where 
    FGet: Fn(&S) -> f64,
    FSet: Fn(&S, f64)
{
    parameter: Parameter,
    get: FGet,
    set: FSet,
    _phantom: PhantomData<S>
}

impl<S, FGet, FSet> ParameterRef<S, FGet, FSet> 
where 
    FGet: Fn(&S) -> f64,
    FSet: Fn(&S, f64)
{
    pub fn float(parameter: FloatParameter, get: FGet, set: FSet) -> Self {
        Self {
            parameter: parameter.into(),
            get, 
            set,
            _phantom: PhantomData,
        }
    }

}

// Description of a parameter
/*pub trait Parameter {
    type Plain;

    fn default(&self) -> Self::Plain;
    fn base(&self) -> Type;
    fn name(&self) -> &str {
        self.base().name()    
    }
    
    fn plain_to_string(&self, plain: Self::Plain) -> String;
    fn string_to_plain(&self, str: &str) -> Result<Self::Plain, ParseError>;
    fn plain_to_normalized(&self, plain: Self::Plain) -> f64;
    fn normalized_to_plain(&self, normalized: f64) -> Self::Plain;
    //fn map<U>(to_plain: impl Fn(&U) -> Self::Plain, from_plain: impl Fn(&Self::Plain) -> U) 
}

impl Parameter for FloatParameter {
    type Plain = f64;

    fn default(&self) -> Self::Plain {
        self.default
    }

    fn base(&self) -> Type {
        Type::Ranged(self)
    }

    fn plain_to_normalized(&self, plain: Self::Plain) -> f64 {
        (plain - self.min) / (self.max - self.min)
    }

    fn normalized_to_plain(&self, normalized: f64) -> Self::Plain {
        self.min + normalized * (self.max - self.min)
    }

    fn plain_to_string(&self, plain: Self::Plain) -> String {
        plain.to_string()
    }

    fn string_to_plain(&self, str: &str) -> Result<Self::Plain, ParseError> {
        str.parse::<f64>().map_err(|_| ParseError)
    }
}

impl Parameter for StringListParameter {
    type Plain = usize;

    fn default(&self) -> Self::Plain {
        self.default_index
    }

    fn base(&self) -> Type {
        Type::StringList(self)
    }

    fn plain_to_normalized(&self, plain: Self::Plain) -> f64 {
        (plain as f64) / (self.string_count() as f64)
    }

    fn normalized_to_plain(&self, normalized: f64) -> Self::Plain {
        todo!()
    }

    fn plain_to_string(&self, plain: Self::Plain) -> String {
        todo!()
    }

    fn string_to_plain(&self, str: &str) -> Result<Self::Plain, ParseError> {
        todo!()
    }
}

pub trait ParameterSequence {

}*/