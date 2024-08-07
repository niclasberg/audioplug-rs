use crate::core::Cursor;

use super::{EventContext, EventStatus, LayoutContext, RenderContext, View, Widget, WidgetNode};

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

    fn build(self, ctx: &mut super::BuildContext) -> Self::Element {
        let widget = self.view.build(ctx);
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
    fn mouse_event(&mut self, event: crate::MouseEvent, ctx: &mut EventContext) -> EventStatus {
        self.widget.mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: crate::event::KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.widget.key_event(event, ctx)
    }

    fn focus_changed(&mut self, has_focus: bool, ctx: &mut EventContext) {
        self.widget.focus_changed(has_focus, ctx)
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

    fn mouse_enter_exit(&mut self, has_mouse_over: bool, ctx: &mut EventContext) -> EventStatus {
        self.widget.mouse_enter_exit(has_mouse_over, ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.widget.render(ctx)
    }
}