use crate::Event;

use super::{EventContext, LayoutContext, RenderContext, View, Widget, WidgetNode};

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
    F: Fn(&mut taffy::Style)
{
    fn event(&mut self, event: Event, ctx: &mut EventContext) {
        self.widget.event(event, ctx)
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        self.widget.layout(inputs, ctx)
    }

    fn style(&self) -> taffy::Style {
        let mut style = self.widget.style();
        (self.style_function)(&mut style);
        style
    }

    fn mouse_enter(&mut self, ctx: &mut EventContext) { 
        self.widget.mouse_enter(ctx)
    }

    fn mouse_exit(&mut self, ctx: &mut EventContext) { 
        self.widget.mouse_exit(ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.widget.render(ctx)
    }

    fn child_count(&self) -> usize { 
        self.widget.child_count()
    }

    fn get_child<'a>(&'a self, i: usize) -> &'a WidgetNode { 
        self.widget.get_child(i)
    }

    fn get_child_mut<'a>(&'a mut self, i: usize) -> &'a mut WidgetNode { 
        self.widget.get_child_mut(i)
    }
}