mod style_builder;
mod length;
mod ui_rect;

use crate::core::Size;
pub use taffy::{FlexDirection, FlexWrap};
pub use style_builder::StyleBuilder;
pub use length::Length;
pub use ui_rect::UiRect;

pub enum Display {
	Block,
	Flex(FlexStyle)
}

pub struct FlexStyle {
	direction: FlexDirection,
	wrap: FlexWrap,
	gap: f64,
}

impl Default for FlexStyle {
	fn default() -> Self {
		Self { 
			direction: Default::default(),
			wrap: Default::default(),
			gap: 0.0
		}
	}
}

impl Default for Display {
	fn default() -> Self {
		Self::Flex(Default::default())
	}
}

pub struct Style {
	pub hidden: bool,
	pub size: Size<Length>,
	pub min_size: Size<Length>,
	pub max_size: Size<Length>,
	pub display: Display,
	pub aspect_ratio: Option<f64>,
	pub padding: UiRect,
}

impl Default for Style {
	fn default() -> Self {
		Self {
			hidden: false,
			size: Default::default(),
			min_size: Default::default(),
			max_size: Default::default(),
			display: Default::default(),
			aspect_ratio: None
		}
	}
}

impl taffy::CoreStyle for Style {
	fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
		if self.hidden {
			taffy::BoxGenerationMode::None
		} else {
			taffy::BoxGenerationMode::Normal
		}
	}

	fn is_block(&self) -> bool {
		match self.display {
			Display::Block => true,
			_ => false
		}
	}

	fn box_sizing(&self) -> taffy::BoxSizing {
		self.style.box_sizing()
	}

	fn overflow(&self) -> taffy::Point<taffy::Overflow> {
		self.style.overflow
	}

	fn scrollbar_width(&self) -> f32 {
		self.style.scrollbar_width
	}

	fn position(&self) -> taffy::Position {
		taffy::Position::Relative
	}

	fn inset(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
		self.style.inset
	}

	fn size(&self) -> taffy::Size<taffy::Dimension> {
		self.style.size
	}

	fn min_size(&self) -> taffy::Size<taffy::Dimension> {
		self.min_size.map(|x| x.into())
	}

	fn max_size(&self) -> taffy::Size<taffy::Dimension> {
		self.style.max_size
	}

	fn aspect_ratio(&self) -> Option<f32> {
		self.aspect_ratio.map(|x| x as _)
	}

	fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
		self.style.margin
	}

	fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
		self.style.padding
	}

	fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
		self.style.border
	}
}

impl taffy::FlexboxContainerStyle for Style {
	fn flex_direction(&self) -> FlexDirection {
		match &self.display {
			Display::Flex(flex) => flex.direction,
			_ => taffy::Style::DEFAULT.flex_direction,
		}
	}

	fn flex_wrap(&self) -> FlexWrap {
		match &self.display {
			Display::Flex(flex) => flex.wrap,
			_ => taffy::Style::DEFAULT.flex_wrap,
		}
	}

	fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
		match &self.display {
			Display::Flex(flex) => taffy::Size::from_length(flex.gap as _),
			_ => taffy::Size::ZERO,
		}
	}

	fn align_content(&self) -> Option<taffy::AlignContent> {
		taffy::Style::DEFAULT.align_content
	}

	fn align_items(&self) -> Option<taffy::AlignItems> {
		taffy::Style::DEFAULT.align_items
	}

	fn justify_content(&self) -> Option<taffy::JustifyContent> {
		taffy::Style::DEFAULT.justify_content
	}
}