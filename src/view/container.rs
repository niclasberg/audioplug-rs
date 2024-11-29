use crate::{app::{BuildContext, ViewContext, Widget}, style::DisplayStyle};

use super::View;

pub struct Container<F> {
	view_factory: F
}

impl<V, F> Container<F> where 
	V: View,
	F: FnOnce(&mut ViewContext) -> V 
{
	pub fn new(view_factory: F) -> Self {
		Self {
			view_factory
		}
	}
}

impl<V, F> View for Container<F> where 
	V: View,
	F: FnOnce(&mut ViewContext) -> V
{
	type Element = ContainerWidget;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		ctx.add_child_with(self.view_factory);
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