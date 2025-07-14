use std::marker::PhantomData;

use crate::{
    app::{
        accessor::SourceId, effect::BindingState, Accessor, CreateContext, Effect, NodeId,
        ReadContext, Readable, WatchContext,
    },
    param::ParameterId,
};

pub struct ReadSignal<T> {
    pub(super) source_id: SourceId,
    _marker: PhantomData<*const T>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignal<T> {}

impl<T> From<ReadSignal<T>> for Accessor<T> {
    fn from(value: ReadSignal<T>) -> Self {
        Self::ReadSignal(value)
    }
}

impl<T> ReadSignal<T> {
    pub(super) fn from_node(node_id: NodeId) -> Self {
        Self {
            source_id: SourceId::Node(node_id),
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_parameter(parameter_id: ParameterId) -> Self {
        Self {
            source_id: SourceId::Parameter(parameter_id),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> ReadSignal<T> {
    pub fn watch(
        self,
        cx: &mut dyn CreateContext,
        mut f: impl FnMut(&mut dyn WatchContext, &T) + 'static,
    ) -> Effect {
        let owner = cx.owner();
        let id = cx.runtime_mut().create_binding_node(
            self.source_id,
            BindingState::new(move |cx| match self.source_id {
                SourceId::Parameter(parameter_id) => {
                    let value = cx
                        .runtime()
                        .get_parameter_ref(parameter_id)
                        .value_as()
                        .unwrap();
                    f(cx, &value);
                }
                SourceId::Node(node_id) => {
                    cx.runtime_mut().update_if_necessary(node_id);
                    if let Some(node) = cx.runtime_mut().lease_node(node_id) {
                        let value = node
                            .get_value_ref()
                            .downcast_ref()
                            .expect("Node should have the correct value type");
                        f(cx, value);
                        cx.runtime_mut().unlease_node(node_id, node);
                    }
                }
            }),
            owner,
        );

        Effect { id }
    }
}

impl<T: 'static> Readable for ReadSignal<T> {
    type Value = T;

    fn get_source_id(&self) -> SourceId {
        self.source_id
    }

    fn track(&self, cx: &mut dyn ReadContext) {
        let scope = cx.scope();
        match self.source_id {
            SourceId::Parameter(parameter_id) => {
                cx.runtime_mut().track_parameter(parameter_id, scope)
            }
            SourceId::Node(node_id) => cx.runtime_mut().track(node_id, scope),
        }
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        match self.source_id {
            SourceId::Parameter(parameter_id) => {
                let value = cx
                    .runtime()
                    .get_parameter_ref(parameter_id)
                    .value_as()
                    .unwrap();
                f(&value)
            }
            SourceId::Node(node_id) => {
                cx.runtime_mut().update_if_necessary(node_id);
                let value = cx
                    .runtime_mut()
                    .get_node_value_ref(node_id)
                    .unwrap()
                    .downcast_ref()
                    .expect("Node should have the correct value type");
                f(value)
            }
        }
    }
}
