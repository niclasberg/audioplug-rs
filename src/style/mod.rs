mod style_builder;
mod length;
mod ui_rect;
mod layout_style;

use crate::core::{Color, Size};
pub use taffy::{FlexDirection, FlexWrap, Overflow};
pub use style_builder::StyleBuilder;
pub use length::Length;
pub use ui_rect::UiRect;
pub(crate) use layout_style::LayoutStyle;

pub trait Measure {
	fn measure(&self, 
		style: &Style,
		width: Option<f64>, 
		height: Option<f64>, 
		available_width: taffy::AvailableSpace, 
		available_height: taffy::AvailableSpace) -> Size;
}

#[derive(Copy, Clone)]
pub enum DisplayStyle<'a> {
	Block,
	Flex(&'a FlexStyle),
	Leaf(&'a dyn Measure)
}

#[derive(Debug, Copy, Clone)]
pub struct FlexStyle {
	pub direction: FlexDirection,
	pub wrap: FlexWrap,
	pub gap: Length,
}

impl FlexStyle {
	pub const DEFAULT: Self = Self {
		direction: FlexDirection::Row,
		wrap: FlexWrap::NoWrap,
		gap: Length::ZERO
	};
}

impl Default for FlexStyle {
	fn default() -> Self {
		Self::DEFAULT
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
	pub overflow_x: Overflow,
	pub overflow_y: Overflow,
	pub background: Option<Color>,
	pub corner_radius: Size
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
			overflow_x: Overflow::Visible,
			overflow_y: Overflow::Visible,
			background: None,
			corner_radius: Size::ZERO,
		}
	}
}
