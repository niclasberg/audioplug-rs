use slotmap::Key;

use crate::{
    param::ParameterId,
    ui::{TypedWidgetId, Widget, WidgetContext, WidgetMut, WidgetRef},
};

use super::{CreateContext, NodeId, ReactiveContext, ReadContext, WriteContext};
use std::{any::Any, cell::RefCell, rc::Rc};

pub trait EffectContext: ReactiveContext + ReadContext + WriteContext + WidgetContext {
    fn as_watch_context(&mut self) -> &mut dyn WatchContext;
}
impl dyn EffectContext + '_ {
    pub fn widget_ref<W: Widget + ?Sized>(&self, id: TypedWidgetId<W>) -> WidgetRef<'_, W> {
        self.widget_ref_dyn(id.id).unchecked_cast()
    }

    pub fn widget_mut<W: Widget + ?Sized>(&mut self, id: TypedWidgetId<W>) -> WidgetMut<'_, W> {
        self.widget_mut_dyn(id.id).unchecked_cast()
    }
}

pub trait WatchContext: ReactiveContext + WriteContext + WidgetContext {}
impl dyn WatchContext + '_ {
    pub fn widget_ref<W: Widget + ?Sized>(&self, id: TypedWidgetId<W>) -> WidgetRef<'_, W> {
        self.widget_ref_dyn(id.id).unchecked_cast()
    }

    pub fn widget_mut<W: Widget + ?Sized>(&mut self, id: TypedWidgetId<W>) -> WidgetMut<'_, W> {
        self.widget_mut_dyn(id.id).unchecked_cast()
    }
}

pub type EffectFn = dyn FnMut(&mut dyn EffectContext);

pub struct EffectState {
    pub(super) f: Rc<RefCell<EffectFn>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut dyn EffectContext) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f)),
        }
    }
}

pub struct Effect {
    pub(super) id: NodeId,
}

impl Effect {
    pub(super) fn new_empty() -> Self {
        Self { id: NodeId::null() }
    }

    pub fn new(
        cx: &mut dyn CreateContext,
        f: impl FnMut(&mut dyn EffectContext) + 'static,
    ) -> Self {
        let id = super::create_effect_node(
            cx,
            EffectState {
                f: Rc::new(RefCell::new(f)),
            },
            true,
        );
        Self { id }
    }

    pub fn new_with_state<T: Any>(
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut dyn EffectContext, Option<T>) -> T + 'static,
    ) -> Self {
        let mut state: Option<T> = None;
        let id = super::create_effect_node(
            cx,
            EffectState {
                f: Rc::new(RefCell::new(move |cx: &mut dyn EffectContext| {
                    let old_state = state.take();
                    state = Some(f(cx, old_state));
                })),
            },
            true,
        );
        Self { id }
    }

    pub(super) fn watch_parameter<T: 'static>(
        cx: &mut dyn CreateContext,
        parameter_id: ParameterId,
        mut f: impl FnMut(&mut dyn WatchContext, &T) + 'static,
    ) -> Self {
        let id = super::create_parameter_binding_node(
            cx,
            parameter_id,
            BindingState::new(move |cx| {
                let value = cx
                    .app_state_mut()
                    .runtime
                    .get_parameter_ref(parameter_id)
                    .value_as()
                    .unwrap();
                f(cx, &value);
            }),
        );

        Self { id }
    }

    pub(super) fn watch_node<T: 'static>(
        cx: &mut dyn CreateContext,
        node_id: NodeId,
        mut f: impl FnMut(&mut dyn WatchContext, &T) + 'static,
    ) -> Self {
        let id = super::create_node_binding_node(
            cx,
            node_id,
            BindingState::new(move |cx| {
                super::update_if_necessary(cx.app_state_mut(), node_id);
                if let Some(node) = cx.app_state_mut().runtime.lease_node(node_id) {
                    let value = node
                        .get_value_ref()
                        .downcast_ref()
                        .expect("Node should have the correct value type");
                    f(cx, value);
                    cx.app_state_mut().runtime.unlease_node(node_id, node);
                }
            }),
        );

        Self { id }
    }

    pub fn watch<T: 'static>(
        cx: &mut dyn CreateContext,
        value_fn: impl Fn(&mut dyn ReadContext) -> T + 'static,
        mut handler_fn: impl FnMut(&mut dyn WatchContext, &T, Option<&T>) + 'static,
    ) -> Self {
        let mut current_value: Option<T> = None;
        let id = super::create_effect_node(
            cx,
            EffectState {
                f: Rc::new(RefCell::new(move |cx: &mut dyn EffectContext| {
                    let old_value = current_value.take();
                    let new_value = value_fn(cx);
                    handler_fn(cx.as_watch_context(), &new_value, old_value.as_ref());
                    current_value = Some(new_value);
                })),
            },
            true,
        );

        Self { id }
    }

    pub fn dispose(self, cx: &mut dyn ReactiveContext) {
        if !self.id.is_null() {
            cx.app_state_mut().runtime.remove_node(self.id);
        }
    }
}

pub type BindingFn = dyn FnMut(&mut dyn WatchContext);

pub struct BindingState {
    pub f: Rc<RefCell<BindingFn>>,
}

impl BindingState {
    pub fn new(f: impl FnMut(&mut dyn WatchContext) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f)),
        }
    }
}
