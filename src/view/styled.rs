use crate::{app::{Accessor, BuildContext, Widget}, style::{Style, StyleBuilder}};

use super::View;

pub struct Styled<V> {
	pub(super) view: V,
	pub(super) style_builder: StyleBuilder
}

impl<V: View> View for Styled<V> {
	type Element = V::Element;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		let widget = self.view.build(ctx);
		apply_style(self.style_builder.aspect_ratio, ctx, |value, style| { style.aspect_ratio = Some(value); });
		apply_style(self.style_builder.background, ctx, |value, style| { style.background = Some(value); });
		apply_style(self.style_builder.border, ctx, |value, style| { style.border = value; });
		apply_style(self.style_builder.corner_radius, ctx, |value, style| { style.corner_radius = value; });
		apply_style(self.style_builder.height, ctx, |value, style| { style.size.height = value; });
		apply_style(self.style_builder.hidden, ctx, |value, style| { style.hidden = value; });
		apply_style(self.style_builder.min_height, ctx, |value, style| { style.min_size.height = value; });
		apply_style(self.style_builder.min_width, ctx, |value, style| { style.min_size.width = value; });
		apply_style(self.style_builder.max_height, ctx, |value, style| { style.max_size.height = value; });
		apply_style(self.style_builder.max_width, ctx, |value, style| { style.max_size.width = value; });
		apply_style(self.style_builder.padding, ctx, |value, style| { style.padding = value; });
		apply_style(self.style_builder.width, ctx, |value, style| { style.size.width = value; });
		apply_style(self.style_builder.border_color, ctx, |value, style| { style.border_color = Some(value); });
		widget
	}
}

fn apply_style<W: Widget, T: Copy + Clone + 'static>(accessor: Option<Accessor<T>>, ctx: &mut BuildContext<W>, apply_fn: impl Fn(T, &mut Style) + 'static + Copy) {
	if let Some(accessor) = accessor {
		let value = ctx.get_and_track(accessor, move|value, mut widget| {
			widget.update_style(|style| apply_fn(value, style));
			widget.request_layout();
		});
		ctx.update_style(|style| apply_fn(value, style));
	}
}