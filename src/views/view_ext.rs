use crate::{
    KeyEvent,
    ui::{
        BuildContext, EventStatus, OverlayOptions, View, ViewProp,
        reactive::{ReactiveValue, WriteContext},
    },
};

use super::key_down::OnKeyEvent;

pub trait ViewExt {
    fn on_key_event<F>(self, f: F) -> OnKeyEvent<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static;
    fn overlay<V2>(self, options: impl Into<ViewProp<OverlayOptions>>, v: V2) -> Overlay<Self, V2>
    where
        Self: Sized,
        V2: View + Sized;
}

impl<V: View + Sized> ViewExt for V {
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
        options: impl Into<ViewProp<OverlayOptions>>,
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

pub struct Overlay<V, V2> {
    parent_view: V,
    child_view: V2,
    options: ViewProp<OverlayOptions>,
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
