use crate::{core::Size, event::{AnimationFrame, KeyEvent}, MouseEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Resize {
        new_size: Size
    },
    Focused,
    Unfocused,
    Animation(AnimationFrame),
    Mouse(MouseEvent),
    Key(KeyEvent)
}