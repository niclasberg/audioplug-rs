use std::{borrow::Borrow, ops::Deref};

use taffy::{FlexDirection, FlexWrap};

use super::{Length, Style};
use crate::core::Size;

pub trait Measure {
    fn measure(
        &self,
        style: &Style,
        width: Option<f64>,
        height: Option<f64>,
        available_width: taffy::AvailableSpace,
        available_height: taffy::AvailableSpace,
    ) -> Size;
}

#[derive(Copy, Clone)]
pub enum DisplayStyle<'a> {
    Block,
    Flex(&'a FlexStyle),
	Grid(&'a GridStyle),
    Leaf(&'a dyn Measure),
}

pub enum OwnedDisplayStyle {
	Block,
    Flex(FlexStyle),
	Grid(GridStyle),
    Leaf(Box<dyn Measure>),
}

impl OwnedDisplayStyle {
	pub fn as_ref(&self) -> DisplayStyle {
		match self {
			OwnedDisplayStyle::Block => DisplayStyle::Block,
			OwnedDisplayStyle::Flex(flex_style) => DisplayStyle::Flex(flex_style),
			OwnedDisplayStyle::Grid(grid_style) => DisplayStyle::Grid(grid_style),
			OwnedDisplayStyle::Leaf(measure) => DisplayStyle::Leaf(measure.deref()),
		}
	}
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
        gap: Length::ZERO,
    };
}

impl Default for FlexStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GridStyle {

}