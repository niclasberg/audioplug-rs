use crate::core::{Point, Size, Vector};

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

impl MouseEvent {
    pub fn with_offset(&self, offset: Vector) -> Self {
        match self {
            MouseEvent::Down { button, position } => MouseEvent::Down { button: *button, position: *position - offset },
            MouseEvent::Up { button, position } => MouseEvent::Up { button: *button, position: *position - offset },
            MouseEvent::DoubleClick { button, position } => MouseEvent::DoubleClick { button: *button, position: *position - offset },
            MouseEvent::Moved { position } => MouseEvent::Moved { position: *position - offset },
            other => *other
        }
    }

    pub fn position(&self) -> Option<Point> {
        match self {
            MouseEvent::Down { position, .. } => Some(*position),
            MouseEvent::Up { position, .. } => Some(*position),
            MouseEvent::DoubleClick { position, .. } => Some(*position),
            MouseEvent::Enter => None,
            MouseEvent::Exit => None,
            MouseEvent::Moved { position } => Some(*position),
        }
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