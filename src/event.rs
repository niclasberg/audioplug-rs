use crate::core::{Point, Size};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    LEFT,
    MIDDLE,
    RIGHT
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent {
    Down {
        button: MouseButton,
        position: Point
    },
    Up {
        button: MouseButton,
        position: Point
    },
    DoubleClick {
        button: MouseButton,
        position: Point
    },
    Enter,
    Exit,
    Moved {
        position: Point
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WindowEvent {
    Resize {
        new_size: Size
    },
    Focused,
    Unfocused
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    Mouse(MouseEvent),
    Window(WindowEvent)
}