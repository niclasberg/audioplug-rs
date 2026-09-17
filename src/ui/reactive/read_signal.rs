use std::marker::PhantomData;

use super::{CanCreate, CanRead, Effect, NodeId, ReactiveValue, WatchContext};
use crate::{
    param::{ParamRef, ParameterId},
    ui::{ViewProp, reactive::WidgetStatus},
};

enum ReadSignalSource<T> {
    Node(NodeId),
    Parameter {
        id: ParameterId,
        getter: fn(ParamRef) -> T,
    },
    WidgetStatus(WidgetStatus<T>),
}

impl<T> Clone for ReadSignalSource<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignalSource<T> {}

pub struct ReadSignal<T> {
    source: ReadSignalSource<T>,
    // Disable Send + Sync
    _phantom: PhantomData<*const T>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReadSignal<T> {}

impl<T> From<ReadSignal<T>> for ViewProp<T> {
    fn from(value: ReadSignal<T>) -> Self {
        Self::ReadSignal(value)
    }
}

impl<T> ReadSignal<T> {
    pub(super) fn from_node(node_id: NodeId) -> Self {
        Self {
            source: ReadSignalSource::Node(node_id),
            _phantom: PhantomData,
        }
    }

    pub(crate) fn from_parameter(id: ParameterId, getter: fn(ParamRef) -> T) -> Self {
        Self {
            source: ReadSignalSource::Parameter { id, getter },
            _phantom: PhantomData,
        }
    }

    pub(crate) fn from_widget_status(signal: WidgetStatus<T>) -> Self {
        Self {
            source: ReadSignalSource::WidgetStatus(signal),
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> ReactiveValue for ReadSignal<T> {
    type Value = T;

    fn track<'cx>(&self, cx: &mut impl CanRead<'cx>) {
        match &self.source {
            ReadSignalSource::Parameter { id, .. } => cx.read_context().track_parameter(*id),
            ReadSignalSource::Node(node_id) => cx.read_context().track(*node_id),
            ReadSignalSource::WidgetStatus(status) => status.track(cx),
        }
    }

    fn with_ref_untracked<'cx, R>(
        &self,
        cx: &mut impl CanRead<'cx>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        let mut cx = cx.read_context();
        match self.source {
            ReadSignalSource::Parameter { id, getter } => {
                let value = getter(cx.reactive_graph.get_parameter_ref(id));
                f(&value)
            }
            ReadSignalSource::Node(node_id) => {
                cx.update_value_if_needed(node_id);
                let value = cx
                    .get_node_value_ref(node_id)
                    .unwrap()
                    .downcast_ref()
                    .expect("Node should have the correct value type");
                f(value)
            }
            ReadSignalSource::WidgetStatus(signal) => signal.with_ref_untracked(&mut cx, f),
        }
    }

    fn watch<'a, F>(self, cx: &mut impl CanCreate<'a>, f: F) -> Effect
    where
        F: FnMut(&mut WatchContext, &Self::Value) + 'static,
    {
        match self.source {
            ReadSignalSource::Parameter { id, getter } => {
                Effect::watch_parameter(cx.create_context(), id, getter, f)
            }
            ReadSignalSource::Node(node_id) => Effect::watch_node(cx.create_context(), node_id, f),
            ReadSignalSource::WidgetStatus(status) => status.watch(cx, f),
        }
    }
}
