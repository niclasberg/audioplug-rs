use std::{any::Any, cell::{Cell, RefCell}, rc::Rc};
use super::{TypedWidgetId, AppState, CreateContext, NodeId, ReactiveContext, ReadContext, Runtime, Scope, Widget, WidgetId, WidgetMut, WidgetRef, WriteContext};

pub struct EffectContext<'a> {
    pub(super) effect_id: NodeId,
    pub(super) app_state: &'a mut AppState
}

impl<'b> EffectContext<'b> {
	pub fn widget_ref<W: Widget>(&self, id: TypedWidgetId<W>) -> WidgetRef<'_, W> {
        WidgetRef::new(&self.app_state, id.id)
    }

    pub fn widget_mut<W: Widget>(&mut self, id: TypedWidgetId<W>) -> WidgetMut<'_, W> {
        WidgetMut::new(&mut self.app_state, id.id)
    }
}

impl<'b> ReactiveContext for EffectContext<'b> {
    fn runtime(&self) -> &Runtime {
        self.app_state.runtime()
    }
    
    fn runtime_mut(&mut self) -> &mut Runtime {
        self.app_state.runtime_mut()
    }
}

impl<'b> ReadContext for EffectContext<'b> {
    fn scope(&self) -> Scope {
        Scope::Node(self.effect_id)
    }
}

impl<'a> WriteContext for EffectContext<'a> {}

pub struct EffectState {
    pub(super) f: Rc<dyn Fn(&mut EffectContext)>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut EffectContext) + 'static) -> Self {
        Self {
            f: Rc::new(f)
        }
    }
}

pub struct Effect {
    
}

impl Effect {
    pub fn new(cx: &mut dyn CreateContext, f: impl Fn(&mut EffectContext) + 'static) -> Self {
		let owner = cx.owner();
        cx.runtime_mut().create_effect_node(EffectState { 
            f: Rc::new(f)
        }, owner, true);
        Self {}
    }

    pub fn new_with_state<T: Any>(cx: &mut dyn CreateContext, f: impl Fn(&mut EffectContext, Option<T>) -> T + 'static) -> Self {
        let state: Cell<Option<T>> = Cell::new(None);
		let owner = cx.owner();
        cx.runtime_mut().create_effect_node(EffectState { 
            f: Rc::new(move |cx: &mut EffectContext| {
                let new_state = f(cx, state.replace(None));
                state.replace(Some(new_state));
            })
        }, owner, true);
        Self {}
    }
}

pub struct BindingState {
    pub f: Rc<RefCell<dyn FnMut(&mut AppState)>>,
}

impl BindingState {
    pub fn new(f: impl FnMut(&mut AppState) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f))
        }
    }
}