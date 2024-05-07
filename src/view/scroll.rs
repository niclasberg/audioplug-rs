use crate::Id;

use super::{View, Widget, WidgetNode};


pub struct Scroll<V> {
    child: V
}

impl<V: View> Scroll<V> {
    pub fn new(child: V) -> Self {
        Self { child }
    }
}

impl<V: View> View for Scroll<V> {
    type Element = ScrollWidget;

    fn build(self, ctx: &mut super::BuildContext) -> Self::Element {
        let child = Box::new(self.child.build(ctx));
        ScrollWidget {
            child
        }
    }
}

pub struct ScrollWidget {
    child: Box<dyn Widget>
}

impl Widget for ScrollWidget {
    fn event(&mut self, event: crate::Event, ctx: &mut super::EventContext) {
        
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut super::LayoutContext) -> taffy::LayoutOutput {
        self.child.layout(inputs, ctx)
    }

    fn style(&self) -> taffy::Style {
        let mut style = self.child.style();
        style.overflow.x = taffy::Overflow::Scroll;
        style.overflow.y = taffy::Overflow::Scroll;
        style
    }

    fn render(&mut self, ctx: &mut super::RenderContext) {
        self.child.render(ctx)
    }
}