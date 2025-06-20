use crate::{
    app::{Accessor, BuildContext, EventStatus, View, WriteContext},
    style::{StyleBuilder, UiRect},
    KeyEvent,
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
