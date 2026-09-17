use taffy::{AlignContent, AlignItems, FlexDirection, FlexWrap};

use super::Length;
use crate::core::Size;

#[derive(Debug, Clone, Copy)]
pub enum AvailableSpace {
    Exact(f32),
    MinContent,
    MaxContent,
}

impl AvailableSpace {
    pub fn unwrap_or(self, value: f32) -> f32 {
        match self {
            Self::Exact(value) => value,
            _ => value,
        }
    }
}

impl From<AvailableSpace> for Option<f32> {
    fn from(val: AvailableSpace) -> Self {
        match val {
            AvailableSpace::Exact(value) => Some(value),
            _ => None,
        }
    }
}

pub trait Measure {
    fn measure(&self, width: AvailableSpace, height: AvailableSpace) -> Size;
}

pub enum LayoutMode<'a> {
    Block,
    Stack,
    Flex(&'a FlexStyle),
    Grid(&'a GridStyle),
    Leaf(&'a mut dyn Measure),
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
        align_content: None,
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

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayStyle {
    Block,
    Stack,
    Flex(FlexStyle),
    Grid(GridStyle),
}

impl Default for DisplayStyle {
    fn default() -> Self {
        Self::Flex(FlexStyle::DEFAULT)
    }
}
