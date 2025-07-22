use std::marker::PhantomData;

use crate::{
    param::ParameterId,
    ui::{Accessor, CreateContext, Effect, NodeId, ReactiveValue, ReadContext, WatchContext},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReadSignalSource {
    Node(NodeId),
    Parameter(ParameterId),
}

pub struct ReadSignal<T> {
    source: ReadSignalSource,
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
            source: ReadSignalSource::Node(node_id),
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_parameter(parameter_id: ParameterId) -> Self {
        Self {
            source: ReadSignalSource::Parameter(parameter_id),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> ReactiveValue for ReadSignal<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        match self.source {
            ReadSignalSource::Parameter(parameter_id) => cx.track_parameter(parameter_id),
            ReadSignalSource::Node(node_id) => cx.track(node_id),
        }
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        match self.source {
            ReadSignalSource::Parameter(parameter_id) => {
                let value = cx
                    .app_state()
                    .runtime
                    .get_parameter_ref(parameter_id)
                    .value_as()
                    .unwrap();
                f(&value)
            }
            ReadSignalSource::Node(node_id) => {
                super::update_if_necessary(cx.app_state_mut(), node_id);
                let value = cx
                    .app_state()
                    .runtime
                    .get_node_value_ref(node_id)
                    .unwrap()
                    .downcast_ref()
                    .expect("Node should have the correct value type");
                f(value)
            }
        }
    }

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn WatchContext, &Self::Value) + 'static,
    {
        match self.source {
            ReadSignalSource::Parameter(parameter_id) => {
                Effect::watch_parameter(cx, parameter_id, f)
            }
            ReadSignalSource::Node(node_id) => Effect::watch_node(cx, node_id, f),
        }
    }
}
