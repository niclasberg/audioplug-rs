use std::{borrow::Borrow, ops::Deref};

use taffy::{AlignContent, AlignItems, FlexDirection, FlexWrap};

use super::{Length, Style};
use crate::core::Size;

#[derive(Debug, Clone, Copy)]
pub enum AvailableSpace {
    Exact(f64),
    MinContent,
    MaxContent,
}

impl AvailableSpace {
    pub fn unwrap_or(self, value: f64) -> f64 {
        match self {
            Self::Exact(value) => value,
            _ => value,
        }
    }
}

impl Into<Option<f64>> for AvailableSpace {
    fn into(self) -> Option<f64> {
        match self {
            Self::Exact(value) => Some(value),
            _ => None,
        }
    }
}

pub trait Measure {
    fn measure(&self, style: &Style, width: AvailableSpace, height: AvailableSpace) -> Size;
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
