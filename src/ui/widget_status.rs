use crate::ui::ReadSignal;
use bitflags::bitflags;

#[derive(Clone, Copy)]
pub struct WidgetStatusSignals {
    pub focused: ReadSignal<bool>,
    pub mouse_hover: ReadSignal<bool>,
    pub mouse_down: ReadSignal<bool>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WidgetStatusFlags: u32 {
        const FOCUSED = 1 << 0;
        const MOUSE_HOVER = 1 << 1;
        const MOUSE_DOWN = 1 << 2;
    }
}
