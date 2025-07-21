use crate::param::{NormalizedValue, ParameterId, ParameterInfo};

pub trait HostHandle {
    fn begin_edit(&self, id: ParameterId);
    fn end_edit(&self, id: ParameterId);
    fn perform_edit(&self, param_info: &dyn ParameterInfo, value: NormalizedValue);
}
