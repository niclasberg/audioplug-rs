use crate::app::{accessor::SourceId, TypedWidgetId, Widget, WidgetContext, WidgetMut, WidgetRef};

use super::{CreateContext, NodeId, ReactiveContext, ReadContext, Readable, WriteContext};
use std::{any::Any, cell::RefCell, rc::Rc};

pub trait EffectContext: ReactiveContext + ReadContext + WriteContext + WidgetContext {}
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

pub(super) type EffectFn = dyn FnMut(&mut dyn EffectContext);

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
    pub fn new(
        cx: &mut dyn CreateContext,
        f: impl FnMut(&mut dyn EffectContext) + 'static,
    ) -> Self {
        let owner = cx.owner();
        let id = cx.runtime_mut().create_effect_node(
            EffectState {
                f: Rc::new(RefCell::new(f)),
            },
            owner,
            true,
        );
        Self { id }
    }

    pub fn new_with_state<T: Any>(
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut dyn EffectContext, Option<T>) -> T + 'static,
    ) -> Self {
        let owner = cx.owner();
        let mut state: Option<T> = None;
        let id = cx.runtime_mut().create_effect_node(
            EffectState {
                f: Rc::new(RefCell::new(move |cx: &mut dyn EffectContext| {
                    let old_state = state.take();
                    state = Some(f(cx, old_state));
                })),
            },
            owner,
            true,
        );
        Self { id }
    }

    pub fn watch<T: 'static>(
        cx: &mut dyn CreateContext,
        source: impl Readable<Value = T> + 'static,
        mut f: impl FnMut(&mut dyn WatchContext, &T) + 'static,
    ) -> Self {
        let owner = cx.owner();
        let source_id = source.get_source_id();
        let id = cx.runtime_mut().create_binding_node(
            source_id,
            BindingState::new(move |cx| match source_id {
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

        Self { id }
    }

    /*pub fn watch_fn<T: 'static>(
        cx: &mut dyn CreateContext,
        value_fn: impl Fn(&mut dyn ReadContext) -> T + 'static,
        mut effect_fn: impl FnMut(&mut dyn WatchContext, T) + 'static,
    ) -> Self {
        let owner = cx.owner();
        let id = cx.runtime_mut().create_effect_node(
            EffectState {
                f: Rc::new(RefCell::new(move |cx: &mut dyn EffectContext| {

                })),
            },
            owner,
            true,
        );

        Self { id }
    }*/

    pub fn dispose(self, cx: &mut dyn ReactiveContext) {
        cx.runtime_mut().remove_node(self.id);
    }
}

pub(super) type BindingFn = dyn FnMut(&mut dyn WatchContext);

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
