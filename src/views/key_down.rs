use crate::{app::{BuildContext, EventContext, EventStatus, RenderContext, View, Widget, WriteContext}, KeyEvent};

pub struct OnKeyEvent<V, F> {
    pub(super) view: V,
    pub(super) on_key_down: F
}

impl<V: View, F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static> View for OnKeyEvent<V, F> {
    type Element = OnKeyEventWidget<V::Element, F>;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        OnKeyEventWidget {
            widget: ctx.build(self.view),
            f: self.on_key_down
        }
    }
}

pub struct OnKeyEventWidget<W, F> {
    widget: W,
    f: F
}

impl<W: Widget, F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static> Widget for OnKeyEventWidget<W, F> {
    fn display_style(&self) -> crate::style::DisplayStyle {
        self.widget.display_style()   
    }

	fn inner_widget(&self) -> Option<&dyn Widget> {
		Some(&self.widget)
	}

	fn inner_widget_mut(&mut self) -> Option<&mut dyn Widget> {
		Some(&mut self.widget)
	}

    fn debug_label(&self) -> &'static str {
        self.widget.debug_label()
    }

    fn render(&mut self, cx: &mut RenderContext) {
        self.widget.render(cx);
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        if self.widget.key_event(event.clone(), ctx) == EventStatus::Handled {
            EventStatus::Handled
        } else {
            (self.f)(ctx.app_state_mut(), event)
        }
    }
}