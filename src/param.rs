use std::{fmt::Display, marker::PhantomData};

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

pub enum Parameter {
    Float(FloatParameter),
    Int(IntParameter),
    StringList(StringListParameter),
    ByPass
}

pub enum ParameterValue {
    Float(f64),
    Int(i64),
    StringList(i64, &'static str),
    ByPass(bool),
}

impl Parameter {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ByPass => "Bypass",
            Self::Float(p) => p.name,
            Self::Int(p) => p.name,
            Self::StringList(StringListParameter{ name, .. }) => name,
        }
    }

    pub fn default(&self) -> f64 {
        match self {
            Parameter::Float(p) => p.default,
            Parameter::Int(p) => p.default as f64,
            Parameter::StringList(p) => p.default_index as f64,
            Parameter::ByPass => 0.0,
        }
    }

    pub fn default_normalized(&self) -> f64 {
        self.normalize(self.default())
    }

    pub fn normalize(&self, value: f64) -> f64 {
        match self {
            Parameter::Float(p) => p.range.normalize(value),
            Parameter::Int(p) => p.range.normalize(value),
            Parameter::StringList(p) => value.clamp(0.0, p.string_count() as f64) / (p.string_count() as f64),
            Parameter::ByPass => value,
        }
    }

    pub fn denormalize(&self, value: f64) -> f64 {
        match self {
            Parameter::Float(_) => todo!(),
            Parameter::Int(_) => todo!(),
            Parameter::StringList(p) => value * (p.string_count() as f64),
            Parameter::ByPass => value,
        }
    }

    pub fn step_count(&self) -> usize {
        match self {
            Self::ByPass => 1,
            Self::Int(p) => p.range.steps(),
            Self::Float(FloatParameter {.. }) => 0,
            Self::StringList(string_list) => string_list.strings.len()
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
}

pub trait ParameterSet<Store> {
    fn init_store(&self) -> Store;
    fn parameter_ref(&self, index: usize) -> &Parameter;
    fn set_normalized(&self, store: &mut Store, index: usize, value: f64);
    fn get_normalized(&self, store: &Store, index: usize) -> f64;
}

impl ParameterSet<Vec<f64>> for Vec<Parameter> {
    fn init_store(&self) -> Vec<f64> {
        let mut store = Vec::with_capacity(self.len());
        for param in self.iter() {
            store.push(param.default());
        }
        store
    }

    fn set_normalized(&self, store: &mut Vec<f64>, index: usize, value: f64) {
        debug_assert!(index < self.len());

        if let Some(param) = self.get(index) {
            store[index] = param.denormalize(value);
        }
    }

    fn get_normalized(&self, store: &Vec<f64>, index: usize) -> f64 {
        debug_assert!(index < self.len());
        
        if let Some(param) = self.get(index) {
            param.normalize(store[index])
        } else {
            0.0
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