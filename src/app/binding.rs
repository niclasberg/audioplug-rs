use std::rc::Rc;

use super::{AppState, Widget, WidgetId, WidgetMut};

pub(super) struct BindingState {
    pub widget_id: WidgetId,
    pub f: Rc<Box<dyn Fn(&mut AppState, WidgetMut<'_, dyn Widget>)>>,
}

impl BindingState {
    pub(super) fn new(widget_id: WidgetId, f: impl Fn(&mut AppState, WidgetMut<'_, dyn Widget>) + 'static) -> Self {
        Self {
            widget_id,
            f: Rc::new(Box::new(f))
        }
    }
}