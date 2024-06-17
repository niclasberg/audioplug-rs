use std::rc::Rc;

use super::SignalContext;

pub(super) struct EffectState {
	pub(super) f: Rc<Box<dyn Fn(&mut dyn SignalContext)>>
}

impl EffectState {
    pub fn new(f: impl Fn(&mut dyn SignalContext) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f))
        }
    }
}