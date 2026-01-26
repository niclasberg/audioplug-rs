use crate::{
    MouseEvent,
    core::{Cursor, PhysicalSize, Point, Rect, ScaleFactor, Size, WindowTheme},
    event::{AnimationFrame, KeyEvent},
    platform::Handle,
};

pub trait WindowHandler: 'static {
    fn init(&mut self, handle: Handle);
    fn event(&mut self, event: WindowEvent);
    fn paint(&mut self, dirty_rect: Rect);
    fn get_cursor(&self, point: Point) -> Option<Cursor>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Resize {
        logical_size: Size,
        physical_size: PhysicalSize,
    },
    Focused,
    Unfocused,
    MouseEnter,
    MouseExit,
    Animation(AnimationFrame),
    Mouse(MouseEvent),
    MouseCaptureEnded,
    Key(KeyEvent),
    ScaleFactorChanged(ScaleFactor),
    ThemeChanged(WindowTheme),
}
