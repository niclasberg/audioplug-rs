use std::{any::Any, cell::Cell, rc::Rc};
use super::{AppState, CreateContext, NodeId, ReactiveContext, ReadContext, Runtime, Scope, Widget, WidgetId, WidgetMut, WidgetRef, WriteContext};

pub struct EffectContext<'a> {
    pub(super) effect_id: NodeId,
    pub(super) app_state: &'a mut AppState
}

impl<'b> EffectContext<'b> {
	pub fn widget_ref(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&self.app_state, id)
    }

    pub fn widget_mut(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(&mut self.app_state, id)
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

pub struct Effect {
    
}

impl Effect {
    pub fn new(cx: &mut dyn CreateContext, f: impl Fn(&mut EffectContext) + 'static) -> Self {
		let owner = cx.owner();
        cx.runtime_mut().create_effect_node(EffectState { 
            f: Rc::new(f)
        }, owner);
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
        }, owner);
        Self {}
    }
}