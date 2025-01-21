use std::rc::Rc;

use super::AppState;

pub struct BindingState {
    pub f: Rc<dyn Fn(&mut AppState)>,
}

impl BindingState {
    pub fn new(f: impl Fn(&mut AppState) + 'static) -> Self {
        Self {
            f: Rc::new(f)
        }
    }
}