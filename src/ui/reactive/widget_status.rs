use crate::ui::{Accessor, AppState, ReactiveValue, ReadSignal, WidgetId};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WidgetStatusFlags: u32 {
        const FOCUSED = 1 << 0;
        const MOUSE_HOVER = 1 << 1;
        const MOUSE_DOWN = 1 << 2;
    }
}

#[derive(Clone, Copy)]
pub struct WidgetStatus<T> {
    pub mask: WidgetStatusFlags,
    pub getter: fn(&AppState, WidgetId) -> T,
}

impl<T> WidgetStatus<T> {
    pub fn as_read_signal(self, widget_id: WidgetId) -> ReadSignal<T> {
        ReadSignal::from_widget_status(widget_id, self.getter, self.mask)
    }
}

pub const FOCUS_STATUS: WidgetStatus<bool> = WidgetStatus {
    mask: WidgetStatusFlags::FOCUSED,
    getter: |app_state, widget_id| app_state.widget_has_focus(widget_id),
};
