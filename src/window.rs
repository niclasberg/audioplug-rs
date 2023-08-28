use crate::{core::{Rectangle, Color}, widget::Widget};
#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(target_os = "macos")]
use crate::mac as platform;


pub struct Renderer<'a>(pub(crate) &'a mut platform::Renderer<'a>);

impl<'a> Renderer<'a> {
    pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        self.0.draw_rectangle(rect, color, line_width)
    }

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        self.0.fill_rectangle(rect, color);
    }

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius_x: f32, radius_y: f32, color: Color) {
        self.0.fill_rounded_rectangle(rect, radius_x, radius_y, color);
    }
}

pub struct Window(platform::Window);

impl Window {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self(platform::Window::new(widget).unwrap())
    }
}