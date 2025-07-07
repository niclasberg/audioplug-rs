use crate::{
    app::{
        Accessor, BuildContext, CallbackContext, EventStatus, MouseEventContext, View, Widget,
        WrappedWidget, WriteContext,
    },
    style::{StyleBuilder, UiRect},
    KeyEvent, MouseButton, MouseEvent,
};

use super::key_down::OnKeyEvent;

pub trait ViewExt {
    fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self>
    where
        Self: Sized;
    fn on_key_event<F>(self, f: F) -> OnKeyEvent<Self, F>
    where
        Self: Sized,
        F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static;
    fn overlay<V2>(self, insets: impl Into<Accessor<UiRect>>, v: V2) -> Overlay<Self, V2>
    where
        Self: Sized,
        V2: View + Sized;
    fn on_click<F>(self, f: F) -> OnClick<Self, F>
    where
        Self: Sized,
        F: Fn(&mut CallbackContext) + 'static;
}

impl<V: View + Sized> ViewExt for V {
    fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self> {
        let style_builder = f(StyleBuilder::default());
        Styled {
            view: self,
            style_builder,
        }
    }

    fn on_key_event<F>(self, f: F) -> OnKeyEvent<Self, F>
    where
        F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static,
    {
        OnKeyEvent {
            view: self,
            on_key_down: f,
        }
    }

    fn on_click<F>(self, f: F) -> OnClick<Self, F>
    where
        F: Fn(&mut CallbackContext) + 'static,
    {
        OnClick {
            parent_view: self,
            on_click_fn: f,
        }
    }

    fn overlay<V2>(self, insets: impl Into<Accessor<UiRect>>, overlay_view: V2) -> Overlay<Self, V2>
    where
        V2: View + Sized,
    {
        Overlay {
            parent_view: self,
            child_view: overlay_view,
            insets: insets.into(),
        }
    }
}

pub struct Styled<V> {
    pub(super) view: V,
    pub(super) style_builder: StyleBuilder,
}

impl<V: View> View for Styled<V> {
    type Element = V::Element;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = self.view.build(ctx);
        ctx.apply_style(self.style_builder);
        widget
    }
}

pub struct Overlay<V, V2> {
    parent_view: V,
    child_view: V2,
    insets: Accessor<UiRect>,
}

impl<V: View, V2: View> View for Overlay<V, V2> {
    type Element = V::Element;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = cx.build(self.parent_view);
        cx.add_overlay(self.child_view);
        widget
    }
}

pub struct OnClick<V, F> {
    parent_view: V,
    on_click_fn: F,
}

impl<V: View, F: Fn(&mut CallbackContext) + 'static> View for OnClick<V, F> {
    type Element = OnClickWidget<V::Element, F>;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let parent = cx.build(self.parent_view);
        OnClickWidget {
            parent,
            on_click_fn: self.on_click_fn,
        }
    }
}

pub struct OnClickWidget<W, F> {
    parent: W,
    on_click_fn: F,
}

impl<W: Widget, F: Fn(&mut CallbackContext) + 'static> WrappedWidget for OnClickWidget<W, F> {
    type Inner = W;

    fn inner(&self) -> &Self::Inner {
        &self.parent
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.parent
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down {
                button: MouseButton::LEFT,
                position,
                ..
            }
            | MouseEvent::DoubleClick {
                button: MouseButton::LEFT,
                position,
                ..
            } if cx.bounds().contains(position) => {
                cx.capture_mouse();
                EventStatus::Handled
            }
            MouseEvent::Up {
                button: MouseButton::LEFT,
                position,
                ..
            } if cx.has_mouse_capture() => {
                cx.release_capture();
                if cx.bounds().contains(position) {
                    (self.on_click_fn)(&mut cx.as_callback_context());
                }
                EventStatus::Handled
            }
            _ => self.parent.mouse_event(event, cx),
        }
    }
}
