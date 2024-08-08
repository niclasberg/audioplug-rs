use crate::{core::{Point, Rectangle, Size}, event::{AnimationFrame, KeyEvent}, core::Cursor, MouseEvent};

use super::{Handle, HandleRef, RendererRef};

pub trait WindowHandler {
	fn init(&mut self, handle: Handle);
    fn event(&mut self, event: WindowEvent, handle: HandleRef);
    fn render(&mut self, bounds: Rectangle, renderer: RendererRef);
    fn get_cursor(&self, point: Point) -> Option<Cursor>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Resize {
        new_size: Size
    },
    Focused,
    Unfocused,
    MouseEnter,
    MouseExit,
    Animation(AnimationFrame),
    Mouse(MouseEvent),
    Key(KeyEvent)
}