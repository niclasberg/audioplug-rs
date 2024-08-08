use crate::{app::{Accessor, AppState, SignalGet}, core::{Color, Cursor, Point, Rectangle, Shape, Transform}, text::TextLayout, window::WindowState, Id};
use super::{IdPath, View, ViewFlags, Widget, WidgetData, WidgetNode};
use crate::platform;

pub struct WidgetContext<'a> {
    widget_data: &'a mut WidgetData
}

impl<'a> WidgetContext<'a> {
    pub fn request_layout(&mut self) {
        self.widget_data.set_flag(ViewFlags::NEEDS_LAYOUT);
    }

    pub fn request_render(&mut self) {
        self.widget_data.set_flag(ViewFlags::NEEDS_RENDER);
    }
}
