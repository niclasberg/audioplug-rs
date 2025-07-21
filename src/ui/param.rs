use std::marker::PhantomData;

use crate::param::{
    AnyParameter, NormalizedValue, Parameter, ParameterId, ParameterInfo, PlainValue,
};

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
            id: p.info().id(),
            _phantom: PhantomData,
        }
    }

    pub fn info<'a>(&self, cx: &'a mut dyn ParamContext) -> &'a dyn ParameterInfo {
        cx.runtime().get_parameter_ref(self.id).info()
    }

    pub fn begin_edit(&self, ctx: &mut dyn ParamContext) {
        ctx.host_handle().begin_edit(self.id);
    }

    pub fn set_value_normalized(&self, cx: &mut dyn ParamContext, value: NormalizedValue) {
        let param_ref = cx.runtime().get_parameter_ref(self.id);
        param_ref.internal_set_value_normalized(value);
        let info = param_ref.info();
        cx.host_handle().perform_edit(info, value);
        cx.runtime_mut().notify_parameter_subscribers(self.id);
    }

    pub fn set_value_plain(&self, cx: &mut dyn ParamContext, value: PlainValue) {
        let param_ref = cx.runtime().get_parameter_ref(self.id);
        param_ref.internal_set_value_plain(value);
        let info = param_ref.info();
        let value = info.normalize(value);
        cx.host_handle().perform_edit(info, value);
        cx.runtime_mut().notify_parameter_subscribers(self.id);
    }

    pub fn set_value<T>(&self, cx: &mut dyn ParamContext, value: T)
    where
        P: Parameter<T>,
    {
    }

    pub fn end_edit(&self, cx: &mut impl ParamContext) {
        cx.host_handle().end_edit(self.id);
    }
}

/*impl<T: Any> Readable for ParamSignal<T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        SourceId::Parameter(self.id)
    }

    fn track(&self, cx: &mut dyn ReadContext) {
        let scope = cx.scope();
        cx.runtime_mut().track_parameter(self.id, scope);
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        self.track(cx);
        self.with_ref_untracked(cx, f)
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        let param_ref = cx.runtime_mut().get_parameter_ref(self.id);
        let value = param_ref.value_as().unwrap();
        f(&value)
    }

    fn get(&self, cx: &mut dyn ReadContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        self.track(cx);
        let param_ref = cx.runtime().get_parameter_ref(self.id);
        param_ref.value_as().unwrap()
    }

    fn get_untracked(&self, cx: &mut dyn ReactiveContext) -> Self::Value
    where
        Self::Value: Clone,
    {
        let param_ref = cx.runtime().get_parameter_ref(self.id);
        param_ref.value_as().unwrap()
    }
}*/
