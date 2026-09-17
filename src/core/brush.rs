use std::fmt::Debug;

use crate::core::{Color, LinearGradient};

#[derive(Debug, Clone)]
pub enum Paint {
    Solid(Color),
    LinearGradient(LinearGradient),
}

impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

impl From<LinearGradient> for Paint {
    fn from(value: LinearGradient) -> Self {
        Self::LinearGradient(value)
    }
}

#[derive(Clone, Copy)]
pub enum BrushRef<'a> {
    Solid(Color),
    LinearGradient(&'a LinearGradient),
}

impl<'a> From<&'a Paint> for BrushRef<'a> {
    fn from(value: &'a Paint) -> Self {
        match value {
            Paint::Solid(color) => Self::Solid(*color),
            Paint::LinearGradient(linear_gradient) => Self::LinearGradient(linear_gradient),
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
