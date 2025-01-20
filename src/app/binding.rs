use std::rc::Rc;

use super::{AppState, WidgetId};

pub(crate) struct BindingState {
    pub f: Rc<dyn Fn(&mut AppState)>,
}

impl BindingState {
    pub(super) fn new(f: impl Fn(&mut AppState) + 'static) -> Self {
        Self {
            f: Rc::new(f)
        }
    }
}