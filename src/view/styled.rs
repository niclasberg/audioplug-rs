use crate::{app::{BuildContext, EventContext, EventStatus, MouseEventContext, RenderContext, StatusChange, Widget}, core::Cursor};

use super::View;

pub struct Styled<V, F> {
    pub(super) view: V,
    pub(super) style_function: F
}

impl<V, F> View for Styled<V, F>
where
    V: View,
    F: Fn(&mut taffy::Style) + 'static
{
    type Element = StyledWidget<V::Element, F>;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = ctx.build(self.view);
        StyledWidget {
            widget,
            style_function: self.style_function
        }
    }
}

pub struct StyledWidget<W, F> {
    widget: W,
    style_function: F
}

impl<W, F> Widget for StyledWidget<W, F> 
where
    W: Widget,
    F: Fn(&mut taffy::Style) + 'static
{
	fn debug_label(&self) -> &'static str {
		self.widget.debug_label()
	}

    fn mouse_event(&mut self, event: crate::MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        self.widget.mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: crate::event::KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.widget.key_event(event, ctx)
    }

    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        self.widget.measure(style, known_dimensions, available_space)
    }

    fn cursor(&self) -> Option<Cursor> {
        self.widget.cursor()
    }

    fn style(&self) -> taffy::Style {
        let mut style = self.widget.style();
        (self.style_function)(&mut style);
        style
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        self.widget.status_updated(event, ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.widget.render(ctx)
    }
}