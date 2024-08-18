use crate::param::{NormalizedValue, ParameterId};

pub trait HostHandle {
    fn begin_edit(&self, id: ParameterId);
    fn end_edit(&self, id: ParameterId);
    fn perform_edit(&self, id: ParameterId, value: NormalizedValue);
}