use crate::{app::{BuildContext, EventContext, EventStatus, MouseEventContext, RenderContext, StatusChange, Widget}, core::{Color, Cursor}, KeyEvent, MouseEvent};

use super::View;

pub struct Background<V: View> {
    pub(super) view: V,
    pub(super) color: Color,
}
	
impl<V: View> View for Background<V> {
    type Element = BackgroundWidget<V::Element>;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = ctx.build(self.view);
        BackgroundWidget { widget, color: self.color }
    }
}

pub struct BackgroundWidget<W> {
    widget: W,
    color: Color,
}

impl<W: Widget> Widget for BackgroundWidget<W> {
	fn debug_label(&self) -> &'static str {
		self.widget.debug_label()
	}

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        self.widget.mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.widget.key_event(event, ctx)
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        self.widget.status_updated(event, ctx)
    }

    fn cursor(&self) -> Option<Cursor> {
        self.widget.cursor()
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(ctx.local_bounds(), self.color);
        self.widget.render(ctx)
    }

    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        self.widget.measure(style, known_dimensions, available_space)
    }
} 
