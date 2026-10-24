use slotmap::Key;

use crate::{
    param::{ParamRef, ParameterId},
    ui::{
        WidgetId, WidgetMut, WidgetRef, Widgets,
        reactive::{
            CreateContext, ReactiveGraph, ReadContext, ReadScope, WidgetStatus, WriteContext,
            runtime::Source,
        },
        widgets::WidgetHandlers,
    },
};

use super::{CanCreate, CanRead, CanWrite, NodeId};
use std::any::Any;

pub struct EffectContext<'a> {
    pub widgets: &'a mut Widgets,
    pub reactive_graph: &'a mut ReactiveGraph,
    pub handlers: &'a mut WidgetHandlers,
    pub effect_id: NodeId,
    pub track_reads: bool,
}

impl<'s> EffectContext<'s> {
    pub fn widget_ref(&self, widget_id: WidgetId) -> WidgetRef<'_> {
        WidgetRef::new(self.widgets, widget_id)
    }

    pub fn widget_mut(&mut self, widget_id: WidgetId) -> WidgetMut<'_> {
        WidgetMut::new(self.widgets, self.handlers, self.reactive_graph, widget_id)
    }
}

impl<'s> CanRead<'s> for EffectContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        let current_widget = self.reactive_graph.owner_widget(self.effect_id);
        let scope = if self.track_reads {
            ReadScope::Node(self.effect_id)
        } else {
            ReadScope::Untracked
        };

        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope,
            current_widget,
        }
    }
}

impl<'s> CanWrite<'s> for EffectContext<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2,
    {
        WriteContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
        }
    }
}

pub type EffectFn = dyn FnMut(&mut EffectContext);

pub struct EffectState {
    pub(crate) f: Box<EffectFn>,
    /// True if the sources for the effect gets re-evaluated each time it is invoked, false otherwise
    pub(crate) dynamic_sources: bool,
}

pub struct Effect {
    pub(super) id: NodeId,
}

impl Effect {
    pub(crate) fn new_empty() -> Self {
        Self { id: NodeId::null() }
    }

    pub fn new<'cx>(
        cx: &mut impl CanCreate<'cx>,
        f: impl FnMut(&mut EffectContext) + 'static,
    ) -> Self {
        let id = cx.create_context().create_effect_node(Box::new(f), true);
        Self { id }
    }

    pub fn new_with_state<'cx, T: Any>(
        cx: &mut impl CanCreate<'cx>,
        f: impl Fn(&mut EffectContext, Option<T>) -> T + 'static,
    ) -> Self {
        let mut state: Option<T> = None;
        let id = cx.create_context().create_effect_node(
            Box::new(move |cx: &mut EffectContext| {
                let old_state = state.take();
                state = Some(f(cx, old_state));
            }),
            true,
        );
        Self { id }
    }

    pub(super) fn watch_parameter<T: 'static>(
        mut cx: CreateContext,
        id: ParameterId,
        getter: fn(ParamRef) -> T,
        mut f: impl FnMut(&mut EffectContext, &T) + 'static,
    ) -> Self {
        let id = cx.create_watcher(
            Source::Parameter(id),
            Box::new(move |cx| {
                let value = getter(cx.reactive_graph.get_parameter_ref(id));
                f(cx, &value);
            }),
        );

        Self { id }
    }

    pub(super) fn watch_node<T: 'static>(
        mut cx: CreateContext,
        node_id: NodeId,
        mut f: impl FnMut(&mut EffectContext, &T) + 'static,
    ) -> Self {
        let id = cx.create_watcher(
            Source::Node(node_id),
            Box::new(move |cx| {
                cx.reactive_graph
                    .update_value_if_necessary(&cx.widgets, node_id);
                if let Some(node) = cx.reactive_graph.lease_node(node_id) {
                    let value = node
                        .get_value_ref()
                        .downcast_ref()
                        .expect("Node should have the correct value type");
                    f(cx, value);
                    cx.reactive_graph.unlease_node(node);
                }
            }),
        );

        Self { id }
    }

    pub(super) fn watch_widget_status<T: 'static>(
        mut cx: CreateContext,
        widget: WidgetId,
        status: WidgetStatus<T>,
        mut f: impl FnMut(&mut EffectContext, &T) + 'static,
    ) -> Self {
        let id = cx.create_watcher(
            Source::Widget(widget, status.mask),
            Box::new(move |cx| {
                let value = (status.getter)(&cx.widgets, widget);
                f(cx, &value);
            }),
        );

        Self { id }
    }

    pub fn watch<'cx, T: 'static>(
        cx: &mut impl CanCreate<'cx>,
        value_fn: impl Fn(&mut ReadContext) -> T + 'static,
        mut handler_fn: impl FnMut(&mut EffectContext, &T, Option<&T>) + 'static,
    ) -> Self {
        let mut current_value: Option<T> = None;
        let id = cx.create_context().create_effect_node(
            Box::new(move |cx: &mut EffectContext| {
                let old_value = current_value.take();
                let new_value = value_fn(&mut cx.read_context());
                handler_fn(cx, &new_value, old_value.as_ref());
                current_value = Some(new_value);
            }),
            true,
        );

        Self { id }
    }
}
