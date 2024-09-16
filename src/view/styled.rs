use crate::{app::{Accessor, BuildContext, EventContext, EventStatus, MouseEventContext, RenderContext, StatusChange, Widget}, core::Cursor};

use super::View;

enum LengthPercentageAccessor {
	None,
	Length(Accessor<f64>),
	Percent(Accessor<f64>)
}

impl LengthPercentageAccessor {
	fn apply<W: Widget>(self, ctx: &mut BuildContext<W>, f: fn(&mut taffy::Style, taffy::LengthPercentage)) {
		match self {
			LengthPercentageAccessor::None => {},
			LengthPercentageAccessor::Length(accessor) => {
				let value = ctx.get_and_track(accessor, move |value, mut widget| {
					widget.update_style(|style| f(style, taffy::LengthPercentage::Length(*value as _)));
					widget.request_layout();
				});
				ctx.update_style(move |style| {
					f(style, taffy::LengthPercentage::Length(value as _));
				});
			}
			LengthPercentageAccessor::Percent(accessor) => {
				let value = ctx.get_and_track(accessor, move |value, mut widget| {
					widget.update_style(|style| f(style, taffy::LengthPercentage::Percent(*value as _)));
					widget.request_layout();
				});
				ctx.update_style(|style| {
					f(style, taffy::LengthPercentage::Percent(value as _));
				});
			},
		};
	}
}

enum DimensionAccessor {
	None,
	Length(Accessor<f64>),
	Percent(Accessor<f64>),
	Auto
}

impl DimensionAccessor {
	fn apply<W: Widget>(self, ctx: &mut BuildContext<W>, f: fn(&mut taffy::Style, taffy::Dimension)) {
		match self {
			DimensionAccessor::None => {},
			DimensionAccessor::Length(accessor) => {
				let value = ctx.get_and_track(accessor, move |value, mut widget| {
					widget.update_style(|style| f(style, taffy::Dimension::Length(*value as _)));
					widget.request_layout();
				});
				ctx.update_style(move |style| {
					f(style, taffy::Dimension::Length(value as _));
				});
			}
			DimensionAccessor::Percent(accessor) => {
				let value = ctx.get_and_track(accessor, move |value, mut widget| {
					widget.update_style(|style| f(style, taffy::Dimension::Percent(*value as _)));
					widget.request_layout();
				});
				ctx.update_style(|style| {
					f(style, taffy::Dimension::Percent(value as _));
				});
			},
			DimensionAccessor::Auto => {
				ctx.update_style(|style| {
					f(style, taffy::Dimension::Auto);
				});
			}
		};
	}
}


pub struct Styled<V> {
	view: V,
	left: LengthPercentageAccessor,
	right: LengthPercentageAccessor,
	top: LengthPercentageAccessor,
	bottom: LengthPercentageAccessor,
	width: DimensionAccessor,
	height: DimensionAccessor,
	min_width: DimensionAccessor,
	max_width: DimensionAccessor,
	min_height: DimensionAccessor,
	max_height: DimensionAccessor
}

impl<V: View> Styled<V> {
	pub fn new(view: V) -> Self {
		Self {
			view,
			left: LengthPercentageAccessor::None,
			right: LengthPercentageAccessor::None,
			top: LengthPercentageAccessor::None,
			bottom: LengthPercentageAccessor::None,
			width: DimensionAccessor::None,
			height: DimensionAccessor::None,
			min_width: DimensionAccessor::None,
			max_width: DimensionAccessor::None,
			min_height: DimensionAccessor::None,
			max_height: DimensionAccessor::None
		}
	}

	pub fn padding(mut self, value: impl Into<Accessor<f64>>) -> Self {
		let value = value.into();
		self.left = LengthPercentageAccessor::Length(value);
		self.right = LengthPercentageAccessor::Length(value);
		self.top = LengthPercentageAccessor::Length(value);
		self.bottom = LengthPercentageAccessor::Length(value);
		self
	}

	pub fn padding_percent(mut self, value: impl Into<Accessor<f64>>) -> Self {
		let value = value.into();
		self.left = LengthPercentageAccessor::Percent(value);
		self.right = LengthPercentageAccessor::Percent(value);
		self.top = LengthPercentageAccessor::Percent(value);
		self.bottom = LengthPercentageAccessor::Percent(value);
		self
	}

	pub fn padding_left(mut self, left: impl Into<Accessor<f64>>) -> Self {
		self.left = LengthPercentageAccessor::Length(left.into());
		self
	}

	pub fn padding_left_percent(mut self, left: impl Into<Accessor<f64>>) -> Self {
		self.left = LengthPercentageAccessor::Percent(left.into());
		self
	}

	pub fn padding_right(mut self, right: impl Into<Accessor<f64>>) -> Self {
		self.right = LengthPercentageAccessor::Length(right.into());
		self
	}

	pub fn padding_right_percent(mut self, right: impl Into<Accessor<f64>>) -> Self {
		self.right = LengthPercentageAccessor::Percent(right.into());
		self
	}

	pub fn padding_top(mut self, top: impl Into<Accessor<f64>>) -> Self {
		self.top = LengthPercentageAccessor::Length(top.into());
		self
	}

	pub fn padding_top_percent(mut self, top: impl Into<Accessor<f64>>) -> Self {
		self.top = LengthPercentageAccessor::Percent(top.into());
		self
	}

	pub fn padding_bottom(mut self, bottom: impl Into<Accessor<f64>>) -> Self {
		self.bottom = LengthPercentageAccessor::Length(bottom.into());
		self
	}

	pub fn padding_bottom_percent(mut self, bottom: impl Into<Accessor<f64>>) -> Self {
		self.bottom = LengthPercentageAccessor::Percent(bottom.into());
		self
	}

	pub fn width(mut self, width: impl Into<Accessor<f64>>) -> Self {
		self.width = DimensionAccessor::Length(width.into());
		self
	}

	pub fn width_percent(mut self, width: impl Into<Accessor<f64>>) -> Self {
		self.width = DimensionAccessor::Percent(width.into());
		self
	}

	pub fn width_auto(mut self) -> Self {
		self.width = DimensionAccessor::Auto;
		self
	}

	pub fn min_width(mut self, width: impl Into<Accessor<f64>>) -> Self {
		self.min_width = DimensionAccessor::Length(width.into());
		self
	}

	pub fn min_width_percent(mut self, width: impl Into<Accessor<f64>>) -> Self {
		self.min_width = DimensionAccessor::Percent(width.into());
		self
	}

	pub fn min_width_auto(mut self) -> Self {
		self.min_width = DimensionAccessor::Auto;
		self
	}

	pub fn max_width(mut self, width: impl Into<Accessor<f64>>) -> Self {
		self.max_width = DimensionAccessor::Length(width.into());
		self
	}

	pub fn max_width_percent(mut self, width: impl Into<Accessor<f64>>) -> Self {
		self.max_width = DimensionAccessor::Percent(width.into());
		self
	}

	pub fn max_width_auto(mut self) -> Self {
		self.max_width = DimensionAccessor::Auto;
		self
	}

	pub fn height(mut self, height: impl Into<Accessor<f64>>) -> Self {
		self.height = DimensionAccessor::Length(height.into());
		self
	}

	pub fn height_percent(mut self, height: impl Into<Accessor<f64>>) -> Self {
		self.height = DimensionAccessor::Percent(height.into());
		self
	}

	pub fn height_auto(mut self) -> Self {
		self.height = DimensionAccessor::Auto;
		self
	}

	pub fn min_height(mut self, height: impl Into<Accessor<f64>>) -> Self {
		self.min_height = DimensionAccessor::Length(height.into());
		self
	}

	pub fn min_height_percent(mut self, height: impl Into<Accessor<f64>>) -> Self {
		self.min_height = DimensionAccessor::Percent(height.into());
		self
	}

	pub fn min_height_auto(mut self) -> Self {
		self.min_height = DimensionAccessor::Auto;
		self
	}

	pub fn max_height(mut self, height: impl Into<Accessor<f64>>) -> Self {
		self.max_height = DimensionAccessor::Length(height.into());
		self
	}

	pub fn max_height_percent(mut self, height: impl Into<Accessor<f64>>) -> Self {
		self.max_height = DimensionAccessor::Percent(height.into());
		self
	}

	pub fn max_height_auto(mut self) -> Self {
		self.max_height = DimensionAccessor::Auto;
		self
	}
}

impl<V: View> View for Styled<V> {
	type Element = V::Element;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		let widget = self.view.build(ctx);

		self.left.apply(ctx, |style, value| style.padding.left = value);
		self.right.apply(ctx, |style, value| style.padding.right = value);
		self.bottom.apply(ctx, |style, value| style.padding.bottom = value);
		self.top.apply(ctx, |style, value| style.padding.top = value);

		self.width.apply(ctx, |style, value| style.size.width = value);
		self.height.apply(ctx, |style, value| style.size.height = value);
		self.min_width.apply(ctx, |style, value| style.min_size.width = value);
		self.max_width.apply(ctx, |style, value| style.max_size.width = value);
		self.min_height.apply(ctx, |style, value| style.min_size.height = value);
		self.max_height.apply(ctx, |style, value| style.max_size.height = value);

		widget
	}
}

