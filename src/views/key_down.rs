use crate::{
    KeyEvent,
    ui::{BuildContext, CallbackContext, EventContext, EventStatus, View, Widget, WidgetAdapter},
};

pub struct OnKeyEvent<V, F> {
    pub(super) view: V,
    pub(super) on_key_down: F,
}

impl<V: View, F: FnMut(CallbackContext, KeyEvent) -> EventStatus + 'static> View
    for OnKeyEvent<V, F>
{
    type Element = OnKeyEventWidget<V::Element, F>;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        OnKeyEventWidget {
            widget: ctx.build_inner(self.view),
            f: self.on_key_down,
        }
    }
}

pub struct OnKeyEventWidget<W, F> {
    widget: W,
    f: F,
}

impl<W: Widget, F: FnMut(CallbackContext, KeyEvent) -> EventStatus + 'static> WidgetAdapter
    for OnKeyEventWidget<W, F>
{
    type Inner = W;

    fn inner(&self) -> &Self::Inner {
        &self.widget
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.widget
    }

    fn key_event(&mut self, event: KeyEvent, cx: &mut EventContext) -> EventStatus {
        self.widget
            .key_event(event.clone(), cx)
            .or_else(|| (self.f)(cx.as_callback_context(), event))
    }
}
