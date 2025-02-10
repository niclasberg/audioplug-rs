use std::{borrow::Borrow, ops::Deref};

use taffy::{AlignContent, AlignItems, FlexDirection, FlexWrap};

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FlexStyle {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub gap: Length,
    pub align_items: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
}

impl FlexStyle {
    pub const DEFAULT: Self = Self {
        direction: FlexDirection::Row,
        wrap: FlexWrap::NoWrap,
        gap: Length::ZERO,
        align_items: None,
        align_content: None
    };
}

impl Default for FlexStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridStyle {
    pub column_templates: Vec<taffy::TrackSizingFunction>,
    pub row_templates: Vec<taffy::TrackSizingFunction>,
}
