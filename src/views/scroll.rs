use taffy::Overflow;

use crate::app::{Accessor, BuildContext, RenderContext, Widget};

use super::View;

enum Direction {
    Horizontal,
    Vertical
}

pub struct Scroll<V> {
    child: V,
	overflow_x: Accessor<Overflow>,
}

impl<V: View> Scroll<V> {
    pub fn new(child: V) -> Self {
        Self { 
            child,
            overflow_x: Accessor::Const(Overflow::Scroll),
        }
    }
}

impl<V: View> View for Scroll<V> {
    type Element = ScrollWidget<V::Element>;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = cx.build(self.child);

        cx.update_style(|s| {
            s.overflow_x = Overflow::Scroll;
        });

        ScrollWidget {
            scroll_pos_x: None,
            scroll_pos_y: None,
            widget
        }
    }
}

pub struct ScrollWidget<W: Widget> {
    scroll_pos_x: Option<f64>,
    scroll_pos_y: Option<f64>,
    widget: W
}

impl<W: Widget> Widget for ScrollWidget<W> {
	fn debug_label(&self) -> &'static str {
		"Scroll"
	}

    fn render(&mut self, ctx: &mut RenderContext) {
        
    }
    
    fn display_style(&self) -> crate::style::DisplayStyle {
        todo!()
    }
}