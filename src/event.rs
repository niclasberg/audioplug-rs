use crate::core::{Key, Modifiers, Point, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MouseButton {
    LEFT = 1,
    MIDDLE = 2,
    RIGHT = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent {
    Down(MouseDownEvent),
    Up(MouseUpEvent),
    Click(MouseClickEvent),
    Moved(MouseMoveEvent),
    DragStarted,
    DragMoved(MouseDragEvent),
    DragEnded,
    DragCancelled,
    Wheel(MouseWheelEvent),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseDownEvent {
    pub button: MouseButton,
    pub position: Point,
    pub modifiers: Modifiers,
    pub is_double_click: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseUpEvent {
    pub button: MouseButton,
    pub position: Point,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseMoveEvent {
    pub position: Point,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseDragEvent {
    pub position: Point,
    pub modifiers: Modifiers,
    pub delta: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseClickEvent {
    pub position: Point,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseWheelEvent {
    pub delta: Vec2,
    pub position: Point,
    pub modifiers: Modifiers,
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
