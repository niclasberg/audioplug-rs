use std::marker::PhantomData;

use crate::{
    param::{AnyParameter, NormalizedValue, Parameter, ParameterId, PlainValue},
    ui::{
        HostHandle,
        prelude::{CanRead, CanWrite},
    },
};

pub struct ParamSetter<P> {
    pub(super) id: ParameterId,
    _phantom: PhantomData<P>,
}

impl<P> Clone for ParamSetter<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for ParamSetter<P> {}

impl<P: AnyParameter> ParamSetter<P> {
    pub fn new(p: &P) -> Self {
        Self {
            id: p.id(),
            _phantom: PhantomData,
        }
    }

    pub fn info<'a>(&self, cx: impl CanRead<'a>) -> &'a dyn AnyParameter {
        cx.read_context()
            .reactive_graph
            .get_parameter_ref(self.id)
            .info()
    }

    pub fn begin_edit<'cx>(&self, cx: impl CanWrite<'cx>) {
        cx.write_context().host_handle().begin_edit(self.id);
    }

    pub fn set_value_normalized<'cx>(&self, cx: impl CanWrite<'cx>, value: NormalizedValue) {
        let mut write_context = cx.write_context();
        let param_ref = write_context.get_parameter_ref(self.id);
        param_ref.set_value_normalized(value);
        let info = param_ref.info();
        write_context.host_handle().perform_edit(info, value);
        write_context.notify_parameter_subscribers(self.id);
    }

    pub fn set_value_plain<'cx>(&self, cx: impl CanWrite<'cx>, value: PlainValue) {
        let mut write_context = cx.write_context();
        let param_ref = write_context.get_parameter_ref(self.id);
        param_ref.set_value_plain(value);
        let info = param_ref.info();
        let value = info.normalize(value);
        write_context.host_handle().perform_edit(info, value);
        write_context.notify_parameter_subscribers(self.id);
    }

    pub fn end_edit<'cx>(&self, cx: impl CanWrite<'cx>) {
        cx.write_context().host_handle().end_edit(self.id);
    }
}

impl<P: Parameter> ParamSetter<P> {
    pub fn set_value<'cx>(&self, cx: impl CanWrite<'cx>, value: P::Value) {
        let mut write_context = cx.write_context();
        let param_ref = write_context.get_parameter_ref(self.id);
        let param = P::downcast_param_ref(param_ref).expect("Parameter should have correct type");
        let value = param.normalized_value(value);
        self.set_value_normalized(write_context, value);
    }
}
