use super::{
    AppState, CreateContext, NodeId, ReactiveContext, ReadContext, Readable, Runtime, Scope,
    TypedWidgetId, Widget, WidgetMut, WidgetRef, WriteContext,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

pub struct EffectContext<'a> {
    pub(super) effect_id: NodeId,
    pub(super) app_state: &'a mut AppState,
}

impl EffectContext<'_> {
    pub fn widget_ref<W: Widget>(&self, id: TypedWidgetId<W>) -> WidgetRef<'_, W> {
        WidgetRef::new(self.app_state, id.id)
    }

    pub fn widget_mut<W: Widget>(&mut self, id: TypedWidgetId<W>) -> WidgetMut<'_, W> {
        WidgetMut::new(self.app_state, id.id)
    }
}

impl ReactiveContext for EffectContext<'_> {
    fn runtime(&self) -> &Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self.app_state.runtime_mut()
    }
}

impl ReadContext for EffectContext<'_> {
    fn scope(&self) -> Scope {
        Scope::Node(self.effect_id)
    }
}

impl WriteContext for EffectContext<'_> {}

pub struct WatchContext<'a> {
    app_state: &'a mut AppState,
}

impl WatchContext<'_> {
    pub fn widget_ref<W: Widget>(&self, id: TypedWidgetId<W>) -> WidgetRef<'_, W> {
        WidgetRef::new(self.app_state, id.id)
    }

    pub fn widget_mut<W: Widget>(&mut self, id: TypedWidgetId<W>) -> WidgetMut<'_, W> {
        WidgetMut::new(self.app_state, id.id)
    }
}

impl ReactiveContext for WatchContext<'_> {
    fn runtime(&self) -> &Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self.app_state.runtime_mut()
    }
}

impl WriteContext for WatchContext<'_> {}

pub struct EffectState {
    pub(super) f: Rc<dyn Fn(&mut EffectContext)>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut EffectContext) + 'static) -> Self {
        Self { f: Rc::new(f) }
    }
}

pub struct Effect {}

impl Effect {
    pub fn new(cx: &mut dyn CreateContext, f: impl Fn(&mut EffectContext) + 'static) -> Self {
        let owner = cx.owner();
        cx.runtime_mut()
            .create_effect_node(EffectState { f: Rc::new(f) }, owner, true);
        Self {}
    }

    pub fn new_with_state<T: Any>(
        cx: &mut dyn CreateContext,
        f: impl Fn(&mut EffectContext, Option<T>) -> T + 'static,
    ) -> Self {
        let state: Cell<Option<T>> = Cell::new(None);
        let owner = cx.owner();
        cx.runtime_mut().create_effect_node(
            EffectState {
                f: Rc::new(move |cx: &mut EffectContext| {
                    let new_state = f(cx, state.replace(None));
                    state.replace(Some(new_state));
                }),
            },
            owner,
            true,
        );
        Self {}
    }

    pub fn watch<T: Clone + 'static>(
        cx: &mut dyn CreateContext,
        source: impl Readable<Value = T> + 'static,
        f: impl Fn(&mut WatchContext, T) + 'static,
    ) -> Self {
        let owner = cx.owner();
        let source_id = source.get_source_id();
        cx.runtime_mut().create_binding_node(
            source_id,
            BindingState {
                f: Rc::new(RefCell::new(move |app_state: &mut AppState| {
                    let value = source.get(app_state);
                    let mut cx = WatchContext { app_state };
                    f(&mut cx, value);
                })),
            },
            owner,
        );

        Self {}
    }
}

pub(super) type BindingFn = dyn FnMut(&mut AppState);

pub struct BindingState {
    pub f: Rc<RefCell<BindingFn>>,
}

impl BindingState {
    pub fn new(f: impl FnMut(&mut AppState) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f)),
        }
    }
}
