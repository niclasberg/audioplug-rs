use std::{any::Any, ops::{Deref, DerefMut}};

use crate::{app::{BuildContext, EventContext, LayoutContext, MouseEventContext, RenderContext}, core::{Color, Cursor}, KeyEvent, MouseEvent}; 

mod view_node;
mod view_sequence;
pub use crate::id::IdPath;
pub use view_node::*;
pub use view_sequence::*;
mod label;
//mod stack;
mod button;
mod linear_layout;
mod slider;
mod xy_pad;
mod textbox;
mod filled;
mod contexts;
mod styled;
mod scroll;
mod view;
mod checkbox;

pub use button::{Button, ButtonWidget};
pub use linear_layout::{Column, Row};
pub use label::Label;
pub use slider::Slider;
pub use xy_pad::XyPad;
pub use textbox::TextBox;
pub use filled::*;
pub use contexts::*;
pub use styled::*;
pub use scroll::*;
pub use view::*;
pub use checkbox::Checkbox;



#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    Handled,
    Ignored
}

pub trait Widget: Any {
    fn mouse_event(&mut self, _event: MouseEvent, _ctx: &mut MouseEventContext) -> EventStatus {
        EventStatus::Ignored
    }

    fn mouse_enter_exit(&mut self, _has_mouse_over: bool, _ctx: &mut EventContext) -> EventStatus { 
        EventStatus::Ignored
    }

    fn key_event(&mut self, _event: KeyEvent, _ctx: &mut EventContext) -> EventStatus {
        EventStatus::Ignored
    }

    fn focus_changed(&mut self, _has_focus: bool, _ctx: &mut EventContext) {}
    
    fn cursor(&self) -> Option<Cursor> {
        None
    }

    /// Measure the widget. This must be implemented for widgets that do not have any children
    fn measure(&self, _style: &taffy::Style, _known_dimensions: taffy::Size<Option<f32>>, _available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        taffy::Size::ZERO
    }

    fn style(&self) -> taffy::Style;
    fn render(&mut self, ctx: &mut RenderContext);

    fn as_any(&self) -> &dyn Any {
        &self
    }
}

impl Widget for Box<dyn Widget> {
    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        self.deref_mut().mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.deref_mut().key_event(event, ctx)
    }

    fn focus_changed(&mut self, has_focus: bool, ctx: &mut EventContext) {
        self.deref_mut().focus_changed(has_focus, ctx)
    }

    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        self.deref_mut().measure(style, known_dimensions, available_space)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.deref_mut().render(ctx)
    }

    fn mouse_enter_exit(&mut self, has_mouse_over: bool, ctx: &mut EventContext) -> EventStatus { 
        self.deref_mut().mouse_enter_exit(has_mouse_over, ctx)
    }

    fn cursor(&self) -> Option<Cursor> {
        self.deref().cursor()
    }

    fn style(&self) -> taffy::Style {
        self.deref().style()
    }
}

pub struct Background<V: View> {
    view: V,
    color: Color,
}

impl<V: View> View for Background<V> {
    type Element = BackgroundWidget<V::Element>;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        let widget = self.view.build(ctx);
        BackgroundWidget { widget, color: self.color }
    }
}

pub struct BackgroundWidget<W> {
    widget: W,
    color: Color,
}

impl<W: Widget> Widget for BackgroundWidget<W> {
    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        self.widget.mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.widget.key_event(event, ctx)
    }

    fn focus_changed(&mut self, has_focus: bool, ctx: &mut EventContext) {
        self.widget.focus_changed(has_focus, ctx)
    }

    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        self.widget.measure(style, known_dimensions, available_space)
    }

    fn cursor(&self) -> Option<Cursor> {
        self.widget.cursor()
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(ctx.local_bounds(), self.color);
        self.widget.render(ctx)
    }

    fn mouse_enter_exit(&mut self, has_mouse_over: bool, ctx: &mut EventContext) -> EventStatus { 
        self.widget.mouse_enter_exit(has_mouse_over, ctx)
    }

    fn style(&self) -> taffy::Style {
        self.widget.style()
    }
} 
