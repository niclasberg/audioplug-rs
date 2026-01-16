use std::cell::Cell;

use crate::{
    param::{ParamRef, Parameter, ParseError},
    ui::ReadSignal,
};

use super::{AnyParameter, NormalizedValue, ParameterId, PlainValue};

pub struct ByPassParameter {
    id: ParameterId,
    value: Cell<bool>,
}

impl ByPassParameter {
    pub fn new(id: ParameterId) -> Self {
        Self {
            id,
            value: Cell::new(false),
        }
    }

    pub fn value(&self) -> bool {
        self.value.get()
    }

    pub fn set_value(&self, value: bool) {
        self.value.replace(value);
    }

    pub fn as_signal(&self) -> ReadSignal<bool> {
        ReadSignal::from_parameter(self.id, |param_ref| match param_ref {
            ParamRef::ByPass(p) => p.value(),
            _ => unreachable!(),
        })
    }
}

impl super::private::Sealed for ByPassParameter {}

impl AnyParameter for ByPassParameter {
    fn id(&self) -> ParameterId {
        self.id
    }

    fn name(&self) -> &str {
        "Bypass"
    }

    fn default_value_plain(&self) -> super::PlainValue {
        PlainValue::from_bool(false)
    }

    fn min_value(&self) -> PlainValue {
        PlainValue::new(0.0)
    }

    fn max_value(&self) -> PlainValue {
        PlainValue::new(1.0)
    }

    fn normalize(&self, value: PlainValue) -> NormalizedValue {
        NormalizedValue(value.0)
    }

    fn denormalize(&self, value: super::NormalizedValue) -> PlainValue {
        PlainValue(value.0)
    }

    fn step_count(&self) -> usize {
        1
    }

    fn value_from_string(&self, str: &str) -> Result<NormalizedValue, ParseError> {
        if str.eq_ignore_ascii_case("on") {
            Ok(NormalizedValue::from_bool(true))
        } else if str.eq_ignore_ascii_case("off") {
            Ok(NormalizedValue::from_bool(false))
        } else {
            Err(ParseError)
        }
    }

    fn string_from_value(&self, value: NormalizedValue) -> String {
        if value.into() {
            "On".to_string()
        } else {
            "Off".to_string()
        }
    }
}

impl Parameter for ByPassParameter {
    type Value = bool;

    fn default_value(&self) -> Self::Value {
        false
    }

    fn plain_value(&self, value: bool) -> PlainValue {
        PlainValue::from_bool(value)
    }

    fn normalized_value(&self, value: bool) -> NormalizedValue {
        NormalizedValue::from_bool(value)
    }

    fn value_from_plain(&self, value: PlainValue) -> Self::Value {
        value.into()
    }

    fn value_from_normalized(&self, value: NormalizedValue) -> Self::Value {
        value.into()
    }

    fn downcast_param_ref<'s>(param_ref: ParamRef<'s>) -> Option<&'s Self> {
        match param_ref {
            ParamRef::ByPass(p) => Some(p),
            _ => None,
        }
    }
}
