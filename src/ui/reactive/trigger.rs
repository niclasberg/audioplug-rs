use std::marker::PhantomData;

use super::{CanCreate, CanRead, CanWrite, NodeId};

#[derive(Clone, Copy)]
pub struct Trigger {
    node_id: NodeId,
    _marker: PhantomData<*const ()>,
}

impl Trigger {
    pub fn new<'cx>(cx: &mut impl CanCreate<'cx>) -> Self {
        Self {
            node_id: cx.create_context().create_trigger(),
            _marker: PhantomData,
        }
    }

    pub fn track<'cx>(&self, cx: &mut impl CanRead<'cx>) {
        cx.read_context().track(self.node_id);
    }

    pub fn notify<'cx>(&self, cx: &mut impl CanWrite<'cx>) {
        cx.write_context().notify(self.node_id);
    }
}

/*#[derive(Copy, Clone)]
pub struct DependentField<S, T, R> {
    source_signal: S,
    f: fn(&T) -> &R,
    trigger_id: NodeId,
}

impl<S, T, R> DependentField<S, T, R>
where
    S: Readable<Value = T>,
{
    pub fn new(
        cx: &mut impl CreateContext,
        source_signal: S,
        f: fn(&T) -> &R,
        owner: Option<Owner>,
    ) -> Self {
        let trigger_id = cx.runtime_mut().create_trigger(owner);
        Self {
            source_signal,
            f,
            trigger_id,
        }
    }

    pub fn notify(&self, cx: &mut dyn WriteContext) {
        cx.runtime_mut().notify(self.trigger_id);
    }

    pub fn dispose(self, cx: &mut impl ReactiveContext) {
        cx.runtime_mut().remove_node(self.trigger_id);
    }
}

impl<S, T, B> Readable for DependentField<S, T, B>
where
    S: Readable<Value = T> + 'static,
    T: Any,
    B: Any,
{
    type Value = B;

    fn get_source_id(&self) -> SourceId {
        SourceId::Node(self.trigger_id)
    }

    fn with_ref<R>(&self, cx: &mut dyn ReadContext, f: impl FnOnce(&Self::Value) -> R) -> R {
        self.source_signal.with_ref(cx, move |x| f((self.f)(x)))
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        self.source_signal
            .with_ref_untracked(cx, move |x| f((self.f)(x)))
    }
}*/
