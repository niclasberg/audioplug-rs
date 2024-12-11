use std::rc::Rc;

use super::{AppState, WidgetId};

pub(super) struct BindingState {
    pub widget_id: WidgetId,
    pub f: Rc<Box<dyn Fn(&mut AppState)>>,
}

impl BindingState {
    pub(super) fn new(widget_id: WidgetId, f: impl Fn(&mut AppState) + 'static) -> Self {
        Self {
            widget_id,
            f: Rc::new(Box::new(f))
        }
    }
}