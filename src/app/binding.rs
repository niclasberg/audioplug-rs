use std::rc::Rc;

use super::{ReactiveContext, Widget, WidgetData, WidgetId, WidgetMut};

pub(super) struct BindingState {
    pub widget_id: WidgetId,
    pub f: Rc<Box<dyn Fn(&mut ReactiveContext, &mut Box<dyn Widget>, &mut WidgetData)>>,
}

impl BindingState {
    pub(super) fn new(widget_id: WidgetId, f: impl Fn(&mut ReactiveContext, &mut Box<dyn Widget>, &mut WidgetData) + 'static) -> Self {
        Self {
            widget_id,
            f: Rc::new(Box::new(f))
        }
    }
}