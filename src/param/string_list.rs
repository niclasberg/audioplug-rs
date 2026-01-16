use std::cell::Cell;

use crate::param::Parameter;

use super::{AnyParameter, NormalizedValue, ParamRef, ParameterId, PlainValue};

pub struct StringListParameter {
    id: ParameterId,
    name: &'static str,
    strings: Vec<String>,
    default_index: usize,
    index: Cell<usize>,
}

impl StringListParameter {
    pub fn new(
        id: ParameterId,
        name: &'static str,
        strings: impl Into<Vec<String>>,
        default_index: usize,
    ) -> Self {
        Self {
            id,
            name,
            strings: strings.into(),
            default_index,
            index: Cell::new(0),
        }
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.strings.iter().position(|x| x == key)
    }

    pub fn value(&self) -> usize {
        self.index.get()
    }

    pub fn set_value(&self, value: usize) {
        self.index.replace(value);
    }
}

impl super::private::Sealed for StringListParameter {}

impl AnyParameter for StringListParameter {
    fn id(&self) -> ParameterId {
        self.id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn default_value_plain(&self) -> PlainValue {
        PlainValue::new(self.default_index as f64)
    }

    fn normalize(&self, value: PlainValue) -> NormalizedValue {
        NormalizedValue(
            value.0.clamp(0.0, self.string_count() as f64) / (self.string_count() as f64),
        )
    }

    fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        PlainValue(value.0 * (self.string_count() as f64))
    }

    fn step_count(&self) -> usize {
        if self.strings.is_empty() {
            0
        } else {
            self.strings.len() - 1
        }
    }

    fn value_from_string(&self, _str: &str) -> Result<NormalizedValue, super::ParseError> {
        todo!()
    }

    fn string_from_value(&self, _value: NormalizedValue) -> String {
        todo!()
    }

    fn min_value(&self) -> PlainValue {
        PlainValue::new(0.0)
    }

    fn max_value(&self) -> PlainValue {
        PlainValue::new(self.step_count() as _)
    }
}

impl Parameter for StringListParameter {
    type Value = usize;

    fn default_value(&self) -> Self::Value {
        self.default_index
    }

    fn plain_value(&self, index: Self::Value) -> PlainValue {
        PlainValue(index as f64)
    }

    fn value_from_plain(&self, value: PlainValue) -> Self::Value {
        todo!()
    }

    fn value_from_normalized(&self, value: NormalizedValue) -> Self::Value {
        todo!()
    }

    fn downcast_param_ref<'s>(param_ref: ParamRef<'s>) -> Option<&'s Self> {
        match param_ref {
            ParamRef::StringList(p) => Some(p),
            _ => None,
        }
    }
}
