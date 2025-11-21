use std::cell::Cell;

use super::{
    AnyParameter, NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue,
};

pub struct StringListParameter {
    index: Cell<usize>,
    info: StringListParameterInfo,
}

impl StringListParameter {}

impl AnyParameter for StringListParameter {
    fn info(&self) -> &dyn ParameterInfo {
        &self.info
    }

    fn plain_value(&self) -> PlainValue {
        PlainValue(self.index.get() as f64)
    }

    fn set_value_normalized(&self, _value: NormalizedValue) {
        todo!()
    }

    fn as_param_ref(&self) -> ParamRef<'_> {
        ParamRef::StringList(self)
    }
}

impl Parameter<usize> for StringListParameter {
    fn value(&self) -> usize {
        self.index.get()
    }

    fn set_value(&self, value: usize) {
        self.index.replace(value);
    }
}

pub struct StringListParameterInfo {
    id: ParameterId,
    name: &'static str,
    strings: Vec<String>,
    default_index: usize,
}

impl StringListParameterInfo {
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
        }
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.strings.iter().position(|x| x == key)
    }
}

impl ParameterInfo for StringListParameterInfo {
    fn id(&self) -> ParameterId {
        self.id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn default_value(&self) -> PlainValue {
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
