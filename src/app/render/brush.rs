use std::fmt::Debug;

use crate::core::{Color, ColorMap, UnitPoint};
pub use crate::platform;

#[derive(Clone)]
pub struct LinearGradient {
    pub(crate) start: UnitPoint,
    pub(crate) end: UnitPoint,
    pub(crate) gradient: platform::NativeLinearGradient,
}

impl LinearGradient {
    pub fn new(color_map: impl Into<ColorMap>, start: UnitPoint, end: UnitPoint) -> Self {
        Self {
            gradient: platform::NativeLinearGradient::new(color_map.into()),
            start,
            end,
        }
    }
}

impl Debug for LinearGradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinearGradient")
            .field("color_map", &self.gradient.color_map)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

pub struct RadialGradient(platform::NativeRadialGradient);

pub struct BitmapBrush {}

#[derive(Debug, Clone)]
pub enum Brush {
    Solid(Color),
    LinearGradient(LinearGradient),
}

impl From<Color> for Brush {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

impl From<LinearGradient> for Brush {
    fn from(value: LinearGradient) -> Self {
        Self::LinearGradient(value)
    }
}

#[derive(Clone, Copy)]
pub enum BrushRef<'a> {
    Solid(Color),
    LinearGradient(&'a LinearGradient),
}

impl<'a> From<&'a Brush> for BrushRef<'a> {
    fn from(value: &'a Brush) -> Self {
        match value {
            Brush::Solid(color) => Self::Solid(*color),
            Brush::LinearGradient(linear_gradient) => Self::LinearGradient(linear_gradient),
        }
    }
}

impl<'a> From<Color> for BrushRef<'a> {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

impl<'a> From<&'a LinearGradient> for BrushRef<'a> {
    fn from(value: &'a LinearGradient) -> Self {
        Self::LinearGradient(value)
    }
}
