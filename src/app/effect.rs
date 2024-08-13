use std::rc::Rc;
use super::ReactiveContext;

pub(super) struct EffectState {
    pub(super) f: Rc<Box<dyn Fn(&mut ReactiveContext)>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut ReactiveContext) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f)),
        }
    }
}
