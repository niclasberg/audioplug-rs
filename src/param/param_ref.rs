use std::any::{Any, TypeId};

use super::{AnyParameter, BoolParameter, ByPassParameter, FloatParameter, IntParameter, NormalizedValue, Parameter, ParameterId, ParameterInfo, PlainValue, StringListParameter};


fn get_parameter_value_as<T: Any, U: Any>(p: &impl Parameter<U>) -> Option<T> {
	if TypeId::of::<T>() == TypeId::of::<PlainValue>() {
		Some(unsafe { std::mem::transmute_copy(&p.plain_value()) })
	} else if TypeId::of::<T>() == TypeId::of::<NormalizedValue>() {
		Some(unsafe { std::mem::transmute_copy(&p.normalized_value()) })
	} else if TypeId::of::<T>() == TypeId::of::<U>() {
		Some(unsafe { std::mem::transmute_copy(&p.value()) })
	} else {
		None
	}
}

pub enum ParamRef<'a> {
    Float(&'a FloatParameter),
    Int(&'a IntParameter),
    StringList(&'a StringListParameter),
    ByPass(&'a ByPassParameter),
    Bool(&'a BoolParameter),
}

impl<'a> ParamRef<'a> {
    pub fn info(&self) -> &'a dyn ParameterInfo {
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

	pub fn value_as<T: Any>(&self) -> Option<T> {
		match self {
            Self::Float(p) => get_parameter_value_as(*p),
            Self::Int(p) => get_parameter_value_as(*p),
            Self::StringList(p) => get_parameter_value_as(*p),
            Self::ByPass(p) => get_parameter_value_as(*p),
            Self::Bool(p) => get_parameter_value_as(*p),
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

