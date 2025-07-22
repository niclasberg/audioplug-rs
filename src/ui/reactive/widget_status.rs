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
