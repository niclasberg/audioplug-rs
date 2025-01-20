use std::{any::Any, cell::Cell, rc::Rc};
use super::{CreateContext, Node, NodeId, ReactiveContext, ReadContext, Runtime, Scope, WriteContext};

pub struct EffectContext<'a> {
    pub(super) effect_id: NodeId,
    pub(super) runtime: &'a mut Runtime
}

impl<'b> ReactiveContext for EffectContext<'b> {
    fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    
    fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
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
    pub fn new(cx: &mut impl CreateContext, f: impl Fn(&mut EffectContext) + 'static) -> Self {
        cx.runtime_mut().create_effect_node(EffectState { 
            f: Rc::new(f)
        });
        Self {}
    }

    pub fn new_with_state<T: Any>(cx: &mut impl CreateContext, f: impl Fn(&mut EffectContext, Option<T>) -> T + 'static) -> Self {
        let state: Cell<Option<T>> = Cell::new(None);
        cx.runtime_mut().create_effect_node(EffectState { 
            f: Rc::new(move |cx: &mut EffectContext| {
                let new_state = f(cx, state.replace(None));
                state.replace(Some(new_state));
            })
        });
        Self {}
    }
}