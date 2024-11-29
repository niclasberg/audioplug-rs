use crate::core::Size;

use super::{DisplayStyle, Style};


/// Style used during layout
pub struct LayoutStyle<'a> {
	pub(crate) style: &'a Style,
	pub(crate) display_style: DisplayStyle<'a>,
	pub(crate) scale_factor: f64,
	pub(crate) window_size: Size<f64>,
}

impl<'a> taffy::CoreStyle for LayoutStyle<'a> {
	fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
		if self.style.hidden {
			taffy::BoxGenerationMode::None
		} else {
			taffy::BoxGenerationMode::Normal
		}
	}

	fn is_block(&self) -> bool {
		match self.display_style {
			DisplayStyle::Block => true,
			_ => false
		}
	}

	fn box_sizing(&self) -> taffy::BoxSizing {
		taffy::BoxSizing::ContentBox
	}

	fn overflow(&self) -> taffy::Point<taffy::Overflow> {
		taffy::Point { x: self.style.overflow_x, y: self.style.overflow_y }
	}

	fn scrollbar_width(&self) -> f32 {
		self.style.scrollbar_width as _
	}

	fn position(&self) -> taffy::Position {
		taffy::Position::Relative
	}

	fn inset(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
		self.style.inset.into()
	}

	fn size(&self) -> taffy::Size<taffy::Dimension> {
		self.style.size.into()
	}

	fn min_size(&self) -> taffy::Size<taffy::Dimension> {
		self.style.min_size.into()
	}

	fn max_size(&self) -> taffy::Size<taffy::Dimension> {
		self.style.max_size.into()
	}

	fn aspect_ratio(&self) -> Option<f32> {
		self.style.aspect_ratio.map(|x| x as _)
	}

	fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
		self.style.margin.into()
	}

	fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
		self.style.padding.into()
	}

	fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
		self.style.border.into()
	}
}

impl<'a> taffy::FlexboxContainerStyle for LayoutStyle<'a> {
	fn flex_direction(&self) -> taffy::FlexDirection {
		match &self.display_style {
			DisplayStyle::Flex(flex) => flex.direction,
			_ => taffy::Style::DEFAULT.flex_direction,
		}
	}

	fn flex_wrap(&self) -> taffy::FlexWrap {
		match &self.display_style {
			DisplayStyle::Flex(flex) => flex.wrap,
			_ => taffy::Style::DEFAULT.flex_wrap,
		}
	}

	fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
		match &self.display_style {
			DisplayStyle::Flex(flex) => taffy::Size { width: flex.gap.into(), height: flex.gap.into() },
			_ => taffy::Style::DEFAULT.gap,
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