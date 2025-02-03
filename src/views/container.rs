use crate::{app::{BuildContext, Widget}, style::DisplayStyle};

use super::View;

pub struct Container<V> {
	view: V
}

impl<V> Container<V> {
	pub fn new(view: V) -> Self {
		Self {
			view
		}
	}
}

impl<V: View> View for Container<V> {
	type Element = ContainerWidget;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		ctx.add_child(self.view);
		ContainerWidget {

		}

	}
}

pub struct ContainerWidget {

}

impl Widget for ContainerWidget {
	fn debug_label(&self) -> &'static str {
		"Container"
	}

	fn render(&mut self, ctx: &mut crate::app::RenderContext) {
		ctx.render_children()
	}

	fn display_style(&self) -> DisplayStyle {
		DisplayStyle::Block
	}
}