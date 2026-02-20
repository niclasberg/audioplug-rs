use crate::ui::{ReadSignal, WidgetId, Widgets};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WidgetStatusFlags: u32 {
        const FOCUSED = 1 << 0;
        const MOUSE_HOVER = 1 << 1;
        const CLICKED = 1 << 2;
    }
}

#[derive(Clone, Copy)]
pub struct WidgetStatus<T> {
    pub mask: WidgetStatusFlags,
    pub getter: fn(&Widgets, WidgetId) -> T,
}

impl<T> WidgetStatus<T> {
    pub fn into_read_signal(self, widget_id: WidgetId) -> ReadSignal<T> {
        ReadSignal::from_widget_status(widget_id, self.getter, self.mask)
    }
}

pub const FOCUS_STATUS: WidgetStatus<bool> = WidgetStatus {
    mask: WidgetStatusFlags::FOCUSED,
    getter: |widgets, widget_id| widgets.widget_has_focus(widget_id),
};

pub const CLICKED_STATUS: WidgetStatus<bool> = WidgetStatus {
    mask: WidgetStatusFlags::CLICKED,
    getter: |widgets, widget_id| widgets.widget_has_captured_mouse(widget_id),
};
