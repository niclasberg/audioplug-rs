use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
};

use crate::{style::DisplayStyle, AnimationFrame, KeyEvent, MouseEvent};

use super::{animation::AnimationContext, EventContext, MouseEventContext, RenderContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    Handled,
    Ignored,
}

impl EventStatus {
    pub fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Handled => Self::Handled,
            Self::Ignored => f(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusChange {
    FocusGained,
    FocusLost,
    MouseEntered,
    MouseExited,
    MouseCaptured,
    MouseCaptureLost,
}

pub trait Widget: Any {
    #[allow(unused_variables)]
    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        EventStatus::Ignored
    }

    #[allow(unused_variables)]
    fn key_event(&mut self, event: KeyEvent, cx: &mut EventContext) -> EventStatus {
        EventStatus::Ignored
    }

    #[allow(unused_variables)]
    fn status_updated(&mut self, event: StatusChange, cx: &mut EventContext) {}

    #[allow(unused_variables)]
    fn animation_frame(&mut self, frame: AnimationFrame, cx: &mut AnimationContext) {}

    fn display_style(&self) -> DisplayStyle;

    /// Widgets that wrap another widget (like background, styled etc) need to implement this method and return the
    /// wrapped widget in order for downcasting to work properly.
    fn inner_widget(&self) -> Option<&dyn Widget> {
        None
    }

    /// Widgets that wrap another widget (like background, styled etc) need to implement this method and return the
    /// wrapped widget in order for downcasting to work properly.
    fn inner_widget_mut(&mut self) -> Option<&mut dyn Widget> {
        None
    }

    fn debug_label(&self) -> &'static str;

    fn render(&mut self, cx: &mut RenderContext);
}

impl dyn Widget + 'static {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            Some(unsafe { &*(self as *const _ as *const T) })
        } else {
            self.inner_widget().and_then(|w| w.downcast_ref())
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if (*self).type_id() == TypeId::of::<T>() {
            Some(unsafe { &mut *(self as *mut _ as *mut T) })
        } else {
            self.inner_widget_mut().and_then(|w| w.downcast_mut())
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

    fn animation_frame(&mut self, frame: AnimationFrame, ctx: &mut AnimationContext) {
        self.deref_mut().animation_frame(frame, ctx);
    }

    fn display_style(&self) -> DisplayStyle {
        self.deref().display_style()
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.deref_mut().render(ctx)
    }

    fn inner_widget(&self) -> Option<&dyn Widget> {
        Some(self.deref())
    }

    fn inner_widget_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(self.deref_mut())
    }
}

pub trait WrappedWidget: Any {
    type Inner: Widget;

    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;
    fn display_style(&self) -> DisplayStyle {
        self.inner().display_style()
    }

    fn debug_label(&self) -> &'static str {
        self.inner().debug_label()
    }

    fn render(&mut self, cx: &mut RenderContext) {
        self.inner_mut().render(cx);
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        self.inner_mut().mouse_event(event, cx)
    }

    fn key_event(&mut self, event: KeyEvent, cx: &mut EventContext) -> EventStatus {
        self.inner_mut().key_event(event, cx)
    }

    fn status_updated(&mut self, event: StatusChange, cx: &mut EventContext) {
        self.inner_mut().status_updated(event, cx);
    }

    fn animation_frame(&mut self, frame: AnimationFrame, cx: &mut AnimationContext) {
        self.inner_mut().animation_frame(frame, cx);
    }
}

impl<T: WrappedWidget> Widget for T {
    fn display_style(&self) -> DisplayStyle {
        self.display_style()
    }

    fn debug_label(&self) -> &'static str {
        self.debug_label()
    }

    fn render(&mut self, cx: &mut RenderContext) {
        self.render(cx);
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        self.mouse_event(event, cx)
    }

    fn key_event(&mut self, event: KeyEvent, cx: &mut EventContext) -> EventStatus {
        self.key_event(event, cx)
    }

    fn status_updated(&mut self, event: StatusChange, cx: &mut EventContext) {
        self.status_updated(event, cx);
    }

    fn animation_frame(&mut self, frame: AnimationFrame, cx: &mut AnimationContext) {
        self.animation_frame(frame, cx);
    }

    fn inner_widget(&self) -> Option<&dyn Widget> {
        Some(self.inner())
    }

    fn inner_widget_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(self.inner_mut())
    }
}
