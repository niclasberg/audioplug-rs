use std::{cell::RefCell, rc::Rc};

use super::AppState;

pub struct BindingState {
    pub f: Rc<RefCell<dyn FnMut(&mut AppState)>>,
}

impl BindingState {
    pub fn new(f: impl FnMut(&mut AppState) + 'static) -> Self {
        Self {
            f: Rc::new(RefCell::new(f))
        }
    }
}