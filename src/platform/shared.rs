use crate::{core::{Cursor, Point, Rectangle, Size, WindowTheme}, event::{AnimationFrame, KeyEvent}, MouseEvent};

use super::{Handle, RendererRef};

pub trait WindowHandler {
	fn init(&mut self, handle: Handle);
    fn event(&mut self, event: WindowEvent);
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
    MouseCaptureEnded,
    Key(KeyEvent),
    ScaleFactorChanged {
        scale_factor: f64
    },
    ThemeChanged(WindowTheme)
}
