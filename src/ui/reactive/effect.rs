use slotmap::Key;

use crate::{
    param::{ParamRef, ParameterId},
    ui::{
        AppState, Widget, WidgetHandle, WidgetId, WidgetMut, WidgetRef, Widgets,
        reactive::{CreateContext, ReadContext, ReadScope, WriteContext},
    },
};

use super::{CanCreate, CanRead, CanWrite, NodeId, WidgetStatusFlags};
use std::{any::Any, cell::RefCell, rc::Rc};

pub struct EffectContext<'a> {
    pub app_state: &'a mut AppState,
    pub effect_id: NodeId,
}

impl<'s> EffectContext<'s> {
    fn as_watch_context(&mut self) -> WatchContext<'_> {
        WatchContext {
            app_state: self.app_state,
        }
    }

    pub fn widget_ref<W: Widget + ?Sized>(
        &self,
        widget_handle: WidgetHandle<W>,
    ) -> WidgetRef<'_, W> {
        WidgetRef::new(
            &self.app_state.widgets,
            &self.app_state.widget_impls,
            widget_handle.id,
        )
    }

    pub fn widget_mut<W: Widget + ?Sized>(
        &mut self,
        widget_handle: WidgetHandle<W>,
    ) -> WidgetMut<'_, W> {
        WidgetMut::new(&mut self.app_state, widget_handle.id)
    }
}

impl<'s> CanRead<'s> for EffectContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        self.app_state.read_context(ReadScope::Node(self.effect_id))
    }
}

impl<'s> CanWrite<'s> for EffectContext<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2,
    {
        self.app_state.write_context()
    }
}

pub struct WatchContext<'a> {
    pub app_state: &'a mut AppState,
}

impl<'a> WatchContext<'a> {
    pub fn widget_ref<W: Widget + ?Sized>(
        &self,
        widget_handle: WidgetHandle<W>,
    ) -> WidgetRef<'_, W> {
        WidgetRef::new(
            &self.app_state.widgets,
            &self.app_state.widget_impls,
            widget_handle.id,
        )
    }

    pub fn widget_mut<W: Widget + ?Sized>(
        &mut self,
        widget_handle: WidgetHandle<W>,
    ) -> WidgetMut<'_, W> {
        WidgetMut::new(&mut self.app_state, widget_handle.id)
    }
}

impl<'s> CanRead<'s> for WatchContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        self.app_state.read_context(ReadScope::Untracked)
    }
}

impl<'s> CanWrite<'s> for WatchContext<'s> {
    fn write_context<'s2>(&'s2 mut self) -> WriteContext<'s2>
    where
        's: 's2,
    {
        self.app_state.write_context()
    }
}

pub type EffectFn = dyn FnMut(&mut EffectContext);

pub struct EffectState {
    pub(super) f: Rc<RefCell<EffectFn>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut EffectContext) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f)),
        }
    }
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
        let id = cx.create_context().create_effect_node(
            EffectState {
                f: Rc::new(RefCell::new(f)),
            },
            true,
        );
        Self { id }
    }

    pub fn new_with_state<'cx, T: Any>(
        cx: &mut impl CanCreate<'cx>,
        f: impl Fn(&mut EffectContext, Option<T>) -> T + 'static,
    ) -> Self {
        let mut state: Option<T> = None;
        let id = cx.create_context().create_effect_node(
            EffectState {
                f: Rc::new(RefCell::new(move |cx: &mut EffectContext| {
                    let old_state = state.take();
                    state = Some(f(cx, old_state));
                })),
            },
            true,
        );
        Self { id }
    }

    pub(super) fn watch_parameter<T: 'static>(
        mut cx: CreateContext,
        id: ParameterId,
        getter: fn(ParamRef) -> T,
        mut f: impl FnMut(&mut WatchContext, &T) + 'static,
    ) -> Self {
        let id = cx.create_parameter_watcher(
            id,
            WatchState::new(move |cx| {
                let value = getter(cx.app_state.reactive_graph.get_parameter_ref(id));
                f(cx, &value);
            }),
        );

        Self { id }
    }

    pub(super) fn watch_node<T: 'static>(
        mut cx: CreateContext,
        node_id: NodeId,
        mut f: impl FnMut(&mut WatchContext, &T) + 'static,
    ) -> Self {
        let id = cx.create_node_watcher(
            node_id,
            WatchState::new(move |cx| {
                cx.app_state
                    .reactive_graph
                    .update_value_if_necessary(&cx.app_state.widgets, node_id);
                if let Some(node) = cx.app_state.reactive_graph.lease_node(node_id) {
                    let value = node
                        .get_value_ref()
                        .downcast_ref()
                        .expect("Node should have the correct value type");
                    f(cx, value);
                    cx.app_state.reactive_graph.unlease_node(node);
                }
            }),
        );

        Self { id }
    }

    pub(super) fn watch_widget_status<T: 'static>(
        mut cx: CreateContext,
        widget: WidgetId,
        status_mask: WidgetStatusFlags,
        value_getter: fn(&Widgets, WidgetId) -> T,
        mut f: impl FnMut(&mut WatchContext, &T) + 'static,
    ) -> Self {
        let id = cx.create_widget_status_watcher(
            widget,
            status_mask,
            WatchState::new(move |cx| {
                let value = value_getter(&cx.app_state.widgets, widget);
                f(cx, &value);
            }),
        );

        Self { id }
    }

    pub fn watch<'cx, T: 'static>(
        cx: &mut impl CanCreate<'cx>,
        value_fn: impl Fn(&mut ReadContext) -> T + 'static,
        mut handler_fn: impl FnMut(&mut WatchContext, &T, Option<&T>) + 'static,
    ) -> Self {
        let mut current_value: Option<T> = None;
        let id = cx.create_context().create_effect_node(
            EffectState {
                f: Rc::new(RefCell::new(move |cx: &mut EffectContext| {
                    let old_value = current_value.take();
                    let new_value =
                        value_fn(&mut cx.app_state.read_context(ReadScope::Node(cx.effect_id)));
                    handler_fn(&mut cx.as_watch_context(), &new_value, old_value.as_ref());
                    current_value = Some(new_value);
                })),
            },
            true,
        );

        Self { id }
    }
}

pub type WatchFn = dyn FnMut(&mut WatchContext);

pub struct WatchState {
    pub f: Rc<RefCell<WatchFn>>,
}

impl WatchState {
    pub fn new(f: impl FnMut(&mut WatchContext) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f)),
        }
    }
}
