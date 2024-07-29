use std::rc::Rc;
use super::AppState;

pub(super) struct EffectState {
    pub(super) f: Rc<Box<dyn Fn(&mut AppState)>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut AppState) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f)),
        }
    }
}
