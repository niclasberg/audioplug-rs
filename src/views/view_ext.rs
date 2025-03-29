use crate::{
    app::{BuildContext, EventStatus, View, WriteContext},
    style::StyleBuilder,
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
        Self: Sized,
        F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static,
    {
        OnKeyEvent {
            view: self,
            on_key_down: f,
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
