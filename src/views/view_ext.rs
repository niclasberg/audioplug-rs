use crate::{
    KeyEvent,
    ui::{
        Accessor, BuildContext, EventStatus, OverlayOptions, ReactiveValue, View, WriteContext,
        style::StyleBuilder,
    },
};

use super::key_down::OnKeyEvent;

pub trait ViewExt {
    fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self>
    where
        Self: Sized;
    fn on_key_event<F>(self, f: F) -> OnKeyEvent<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static;
    fn overlay<V2>(self, options: impl Into<Accessor<OverlayOptions>>, v: V2) -> Overlay<Self, V2>
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
        F: FnMut(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static,
    {
        OnKeyEvent {
            view: self,
            on_key_down: f,
        }
    }

    fn overlay<V2>(
        self,
        options: impl Into<Accessor<OverlayOptions>>,
        overlay_view: V2,
    ) -> Overlay<Self, V2>
    where
        V2: View + Sized,
    {
        Overlay {
            parent_view: self,
            child_view: overlay_view,
            options: options.into(),
        }
    }
}

pub struct Styled<V> {
    pub(super) view: V,
    pub(super) style_builder: StyleBuilder,
}

impl<V: View> View for Styled<V> {
    type Element = V::Element;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = self.view.build(cx);
        cx.apply_style(move |style| style.merge(self.style_builder));
        widget
    }
}

pub struct Overlay<V, V2> {
    parent_view: V,
    child_view: V2,
    options: Accessor<OverlayOptions>,
}

impl<V: View, V2: View> View for Overlay<V, V2> {
    type Element = V::Element;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = cx.build_inner(self.parent_view);
        let options = self.options.get(cx);
        cx.add_overlay(self.child_view, options);
        widget
    }
}
