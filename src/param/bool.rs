use super::{NormalizedValue, ParamRef, Parameter, ParameterId, ParameterInfo, PlainValue};

pub struct BoolParameter {
    info: BoolParameterInfo,
    value: bool
}

impl Parameter<bool> for BoolParameter {
    type Info = BoolParameterInfo;

    fn info(&self) -> &Self::Info {
        &self.info
    }

    fn plain_value(&self) -> PlainValue {
        if self.value { 1.0 } else { 0.0 }
    }

    fn normalized_value(&self) -> NormalizedValue {
        self.plain_value()
    }
	
	fn as_param_ref(&self) -> ParamRef {
		ParamRef::Bool(&self)
	}
    
    fn set_value_normalized(&self, value: NormalizedValue) {
        todo!()
    }
}


pub struct BoolParameterInfo {
    id: ParameterId,
    name: &'static str,
    default: bool
}

impl BoolParameterInfo {
    pub fn new(id: ParameterId, name: &'static str, default: bool) -> Self {
        Self { id, name, default }
    }
}

impl ParameterInfo for BoolParameterInfo {
	fn id(&self) -> ParameterId {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn default_value(&self) -> super::PlainValue {
		let value = if self.default { 1.0 } else { 0.0 };
		PlainValue::new(value)
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
}