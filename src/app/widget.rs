use std::{any::{Any, TypeId}, ops::{Deref, DerefMut}};

use crate::{core::Cursor, KeyEvent, MouseEvent};

use super::{EventContext, MouseEventContext, RenderContext};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    Handled,
    Ignored
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusChange {
    FocusGained,
    FocusLost,
    MouseEntered,
    MouseExited,
    MouseCaptured,
    MouseCaptureLost
}

pub trait Widget: Any {
    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        ctx.forward_to_children(event)
    }

    fn key_event(&mut self, _event: KeyEvent, _ctx: &mut EventContext) -> EventStatus {
        EventStatus::Ignored
    }

    fn status_updated(&mut self, _event: StatusChange, _ctx: &mut EventContext) {}
    
    fn cursor(&self) -> Option<Cursor> {
        None
    }

    /// Measure the widget. This must be implemented for widgets that do not have any children
    fn measure(&self, _style: &taffy::Style, _known_dimensions: taffy::Size<Option<f32>>, _available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        taffy::Size::ZERO
    }

	fn debug_label(&self) -> &'static str;

    fn style(&self) -> taffy::Style;
    fn render(&mut self, ctx: &mut RenderContext);
}

impl dyn Widget + 'static {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            Some(unsafe { &*(self as *const _ as *const T) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if (&*self).type_id() == TypeId::of::<T>() {
            Some(unsafe { &mut *(self as *mut _ as *mut T) })
        } else {
            None
        }
    }
}

impl Widget for Box<dyn Widget> {
	fn debug_label(&self) -> &'static str {
		self.deref().debug_label()
	}

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        self.deref_mut().mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.deref_mut().key_event(event, ctx)
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        self.deref_mut().status_updated(event, ctx)
    }

    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        self.deref().measure(style, known_dimensions, available_space)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.deref_mut().render(ctx)
    }

    fn cursor(&self) -> Option<Cursor> {
        self.deref().cursor()
    }

    fn style(&self) -> taffy::Style {
        self.deref().style()
    }
}