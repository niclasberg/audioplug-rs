use std::fmt::Debug;

use crate::core::{Color, LinearGradient};

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

impl From<Color> for BrushRef<'_> {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

impl<'a> From<&'a LinearGradient> for BrushRef<'a> {
    fn from(value: &'a LinearGradient) -> Self {
        Self::LinearGradient(value)
    }
}
