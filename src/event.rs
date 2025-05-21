use crate::core::{Key, Modifiers, Point, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MouseButton {
    LEFT = 1,
    MIDDLE = 2,
    RIGHT = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent {
    Down {
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    },
    Up {
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    },
    DoubleClick {
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    },
    Moved {
        position: Point,
        modifiers: Modifiers,
    },
    Wheel {
        delta: Vector,
        position: Point,
        modifiers: Modifiers,
    },
}

impl MouseEvent {
    pub fn with_offset(&self, offset: Vector) -> Self {
        match *self {
            MouseEvent::Down {
                button,
                position,
                modifiers,
            } => MouseEvent::Down {
                button,
                position: position - offset,
                modifiers,
            },
            MouseEvent::Up {
                button,
                position,
                modifiers,
            } => MouseEvent::Up {
                button,
                position: position - offset,
                modifiers,
            },
            MouseEvent::DoubleClick {
                button,
                position,
                modifiers,
            } => MouseEvent::DoubleClick {
                button,
                position: position - offset,
                modifiers,
            },
            MouseEvent::Moved {
                position,
                modifiers,
            } => MouseEvent::Moved {
                position: position - offset,
                modifiers,
            },
            MouseEvent::Wheel {
                delta,
                position,
                modifiers,
            } => MouseEvent::Wheel {
                delta,
                position: position - offset,
                modifiers,
            },
        }
    }

    pub fn position(&self) -> Point {
        match self {
            MouseEvent::Down { position, .. } => *position,
            MouseEvent::Up { position, .. } => *position,
            MouseEvent::DoubleClick { position, .. } => *position,
            MouseEvent::Moved { position, .. } => *position,
            MouseEvent::Wheel { position, .. } => *position,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MouseButtons(u8);

impl MouseButtons {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn insert(&mut self, button: MouseButton) {
        self.0 |= 1 << button as u8;
    }

    pub fn remove(&mut self, button: MouseButton) {
        self.0 &= !(1 << button as u8);
    }

    pub fn contains(&self, button: MouseButton) -> bool {
        (self.0 & (1 << button as u8)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyEvent {
    KeyUp {
        key: Key,
        modifiers: Modifiers,
    },
    KeyDown {
        key: Key,
        modifiers: Modifiers,
        str: Option<String>,
        repeat_count: usize,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AnimationFrame {
    pub timestamp: f64,
}
