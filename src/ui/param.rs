use std::marker::PhantomData;

use crate::param::{AnyParameter, NormalizedValue, Parameter, ParameterId, PlainValue};

use super::{HostHandle, ReactiveContext};

pub trait ParamContext: ReactiveContext {
    fn host_handle(&self) -> &dyn HostHandle;
}

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

    pub fn info<'a>(&self, cx: &'a mut dyn ParamContext) -> &'a dyn AnyParameter {
        cx.app_state().runtime.get_parameter_ref(self.id).info()
    }

    pub fn begin_edit(&self, ctx: &mut dyn ParamContext) {
        ctx.host_handle().begin_edit(self.id);
    }

    pub fn set_value_normalized(&self, cx: &mut dyn ParamContext, value: NormalizedValue) {
        let param_ref = cx.app_state().runtime.get_parameter_ref(self.id);
        param_ref.internal_set_value_normalized(value);
        let info = param_ref.info();
        cx.host_handle().perform_edit(info, value);
        super::reactive::notify_parameter_subscribers(cx.app_state_mut(), self.id);
    }

    pub fn set_value_plain(&self, cx: &mut dyn ParamContext, value: PlainValue) {
        let param_ref = cx.app_state().runtime.get_parameter_ref(self.id);
        param_ref.internal_set_value_plain(value);
        let info = param_ref.info();
        let value = info.normalize(value);
        cx.host_handle().perform_edit(info, value);
        super::reactive::notify_parameter_subscribers(cx.app_state_mut(), self.id);
    }

    pub fn end_edit(&self, cx: &mut impl ParamContext) {
        cx.host_handle().end_edit(self.id);
    }
}

impl<P: Parameter> ParamSetter<P> {
    pub fn set_value(&self, cx: &'a mut dyn ParamContext, value: P::Value) {}
}
