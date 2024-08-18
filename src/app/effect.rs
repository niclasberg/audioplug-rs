use std::rc::Rc;
use super::Runtime;

pub(super) struct EffectState {
    pub(super) f: Rc<Box<dyn Fn(&mut Runtime)>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut Runtime) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f)),
        }
    }
}
