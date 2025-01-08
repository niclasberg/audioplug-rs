use std::{any::Any, cell::Cell, rc::Rc};
use super::{Node, NodeId, Path, ReactiveContext, Runtime, SignalCreator};

pub struct EffectContext<'a> {
    pub(super) effect_id: NodeId,
    pub(super) runtime: &'a mut Runtime
}

impl<'b> ReactiveContext for EffectContext<'b> {
	fn track(&mut self, source_id: NodeId) {
		self.runtime.subscriptions.add_node_subscription(source_id, self.effect_id);
	}

	fn track_parameter(&mut self, source_id: crate::param::ParameterId) {
		self.runtime.subscriptions.add_parameter_subscription(source_id, self.effect_id);
	}

    fn get_node_mut<'a>(&'a mut self, signal_id: NodeId, child_path: Path) -> &'a mut Node {
        self.runtime.get_node_mut(signal_id, child_path)
    }

    fn get_parameter_ref<'a>(&'a self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.get_parameter_ref(parameter_id)
    }
}

pub struct EffectState {
    pub(super) f: Rc<Box<dyn Fn(&mut EffectContext)>>,
}

pub struct Effect {
    
}

impl Effect {
    pub fn new(cx: &mut impl SignalCreator, f: impl Fn(&mut EffectContext) + 'static) -> Self {
        cx.create_effect_node(EffectState { 
            f: Rc::new(Box::new(f))
        });
        Self {}
    }

    pub fn new_with_state<T: Any>(cx: &mut impl SignalCreator, f: impl Fn(&mut EffectContext, Option<T>) -> T + 'static) -> Self {
        let state: Cell<Option<T>> = Cell::new(None);
        cx.create_effect_node(EffectState { 
            f: Rc::new(Box::new(move |cx: &mut EffectContext| {
                let new_state = f(cx, state.replace(None));
                state.replace(Some(new_state));
            }))
        });
        Self {}
    }
}