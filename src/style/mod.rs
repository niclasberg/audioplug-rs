mod style_builder;
mod length;
mod ui_rect;
mod layout_style;

use crate::core::Size;
pub use taffy::{FlexDirection, FlexWrap};
pub use style_builder::StyleBuilder;
pub use length::Length;
pub use ui_rect::UiRect;
pub(crate) use layout_style::LayoutStyle;

#[derive(Debug, Copy, Clone)]
pub enum DisplayStyle {
	Block,
	Flex(FlexStyle)
}

#[derive(Debug, Copy, Clone)]
pub struct FlexStyle {
	pub direction: FlexDirection,
	pub wrap: FlexWrap,
	pub gap: Length,
}

impl Default for FlexStyle {
	fn default() -> Self {
		Self { 
			direction: Default::default(),
			wrap: Default::default(),
			gap: Length::ZERO
		}
	}
}

impl Default for DisplayStyle {
	fn default() -> Self {
		Self::Flex(Default::default())
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Style {
	pub hidden: bool,
	pub size: Size<Length>,
	pub min_size: Size<Length>,
	pub max_size: Size<Length>,
	pub aspect_ratio: Option<f64>,
	pub padding: UiRect,
	pub border: UiRect,
	pub margin: UiRect,
	pub inset: UiRect,
	pub scrollbar_width: f64, 
}

impl Default for Style {
	fn default() -> Self {
		Self {
			hidden: false,
			size: Default::default(),
			min_size: Default::default(),
			max_size: Default::default(),
			aspect_ratio: None,
			padding: UiRect::ZERO,
			border: UiRect::ZERO,
			margin: UiRect::ZERO,
			inset: UiRect::ZERO,
			scrollbar_width: 5.0,
		}
	}
}
