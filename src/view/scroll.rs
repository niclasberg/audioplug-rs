use crate::app::{BuildContext, RenderContext, Widget};

use super::View;


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

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let child = Box::new(ctx.build(self.child));
        ScrollWidget {
            child
        }
    }
}

pub struct ScrollWidget {
    child: Box<dyn Widget>
}

impl Widget for ScrollWidget {
	fn debug_label(&self) -> &'static str {
		"Scroll"
	}

    fn measure(&self, style: &taffy::Style, known_dimensions: taffy::Size<Option<f32>>, available_space: taffy::Size<taffy::AvailableSpace>) -> taffy::Size<f32> {
        self.child.measure(style, known_dimensions, available_space)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.child.render(ctx)
    }
}