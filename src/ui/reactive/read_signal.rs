use std::marker::PhantomData;

use crate::{
    param::{ParamRef, ParameterId},
    ui::{
        Accessor, CreateContext, Effect, NodeId, ReactiveValue, ReadContext, WatchContext,
        WidgetId, Widgets, reactive::widget_status::WidgetStatusFlags,
    },
};

enum ReadSignalSource<T> {
    Node(NodeId),
    Parameter {
        id: ParameterId,
        getter: fn(ParamRef) -> T,
    },
    WidgetStatus {
        widget_id: WidgetId,
        value_fn: fn(&Widgets, WidgetId) -> T,
        status_mask: WidgetStatusFlags,
    },
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

impl<T> From<ReadSignal<T>> for Accessor<T> {
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

    pub(crate) fn from_widget_status(
        widget_id: WidgetId,
        value_getter: fn(&Widgets, WidgetId) -> T,
        status_mask: WidgetStatusFlags,
    ) -> Self {
        Self {
            source: ReadSignalSource::WidgetStatus {
                widget_id,
                value_fn: value_getter,
                status_mask,
            },
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> ReactiveValue for ReadSignal<T> {
    type Value = T;

    fn track(&self, cx: &mut dyn ReadContext) {
        match &self.source {
            ReadSignalSource::Parameter { id, .. } => cx.track_parameter(*id),
            ReadSignalSource::Node(node_id) => cx.track(*node_id),
            ReadSignalSource::WidgetStatus {
                widget_id,
                status_mask,
                ..
            } => cx.track_widget_status(*widget_id, *status_mask),
        }
    }

    fn with_ref_untracked<R>(
        &self,
        cx: &mut dyn super::ReactiveContext,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        match self.source {
            ReadSignalSource::Parameter { id, getter } => {
                let value = getter(cx.reactive_graph().get_parameter_ref(id));
                f(&value)
            }
            ReadSignalSource::Node(node_id) => {
                super::update_if_necessary(cx, node_id);
                let value = cx
                    .reactive_graph()
                    .get_node_value_ref(node_id)
                    .unwrap()
                    .downcast_ref()
                    .expect("Node should have the correct value type");
                f(value)
            }
            ReadSignalSource::WidgetStatus {
                widget_id,
                value_fn: value_getter,
                ..
            } => {
                let value = value_getter(cx.widgets(), widget_id);
                f(&value)
            }
        }
    }

    fn watch<F>(self, cx: &mut dyn CreateContext, f: F) -> Effect
    where
        F: FnMut(&mut dyn WatchContext, &Self::Value) + 'static,
    {
        match self.source {
            ReadSignalSource::Parameter { id, getter } => {
                Effect::watch_parameter(cx, id, getter, f)
            }
            ReadSignalSource::Node(node_id) => Effect::watch_node(cx, node_id, f),
            ReadSignalSource::WidgetStatus {
                widget_id,
                value_fn: value_getter,
                status_mask,
            } => Effect::watch_widget_status(cx, widget_id, status_mask, value_getter, f),
        }
    }
}
