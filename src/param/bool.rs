use vst3_sys::vst::ParameterInfo;

use super::{Parameter, ParameterId};



pub struct BoolParameter {
    info: BoolParameterInfo,
    value: bool
}

impl Parameter for BoolParameter {
    type Info = BoolParameterInfo;

    fn info(&self) -> Self::Info {
        &self.info
    }

    fn get_plain_value(&self) -> super::PlainValue {
        todo!()
    }

    fn get_normalized_value(&self) -> super::NormalizedValue {
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
    
}