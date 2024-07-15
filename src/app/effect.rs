use std::rc::Rc;

use super::App;

pub(super) struct EffectState {
    pub(super) f: Rc<Box<dyn Fn(&mut App)>>,
}

impl EffectState {
    pub fn new(f: impl Fn(&mut App) + 'static) -> Self {
        Self {
            f: Rc::new(Box::new(f)),
        }
    }
}
