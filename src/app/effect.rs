use std::{any::Any, rc::Rc};
use super::{NodeId, Runtime, SignalCreator, SignalGetContext};

pub struct EffectContext<'a> {
    pub(super) effect_id: NodeId,
    pub(super) runtime: &'a mut Runtime
}

impl<'b> SignalGetContext for EffectContext<'b> {
    fn get_node_value_ref_untracked<'a>(&'a self, node_id: NodeId) -> &'a dyn Any {
        self.runtime.get_node_value_ref_untracked(node_id)
    }

    fn get_node_value_ref<'a>(&'a mut self, signal_id: NodeId) -> &'a dyn Any {
        self.runtime.add_subscription(signal_id, self.effect_id);
        self.runtime.get_node_value_ref_untracked(signal_id)
    }

    fn get_parameter_ref_untracked<'a>(&'a self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.get_parameter_ref_untracked(parameter_id)
    }

    fn get_parameter_ref<'a>(&'a mut self, parameter_id: crate::param::ParameterId) -> crate::param::ParamRef<'a> {
        self.runtime.add_parameter_subscription(parameter_id, self.effect_id);
        self.runtime.get_parameter_ref_untracked(parameter_id)
    }
}

pub struct EffectState {
    pub(super) f: Rc<Box<dyn Fn(&mut EffectContext)>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut EffectContext) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f)),
        }
    }
}

pub struct Effect {

}

impl Effect {
    pub fn new(cx: &mut impl SignalCreator, f: impl Fn(&mut EffectContext) + 'static) -> Self {
        let id = cx.create_effect_node(EffectState::new(f));
        //self.runtime.notify(&id);
        Self {}
    }
}