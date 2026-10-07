mod alignment;
mod axis;
mod capsule;
mod color;
mod constraint;
mod cursor;
pub mod diff;
mod ellipse;
mod gradient;
mod image;
mod interpolation;
mod keyboard;
mod path;
mod point;
mod poly;
mod rectangle;
mod rounded_rectangle;
mod shape;
mod size;
mod taffy_compat;
mod text;
mod transform;
mod unit_point;
mod vector;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

pub use alignment::{Align, HAlign, VAlign};
pub use axis::Axis;
pub use capsule::Capsule;
pub use color::Color;
pub use constraint::*;
pub use cursor::Cursor;
pub use ellipse::{Circle, Ellipse};
pub use gradient::*;
pub use image::ImageData;
use indexmap::{IndexMap, IndexSet};
pub use interpolation::{Lerp, SpringPhysics, SpringProperties};
pub use keyboard::{Key, Modifiers};
pub use path::{CubicBezier, FillRule, Line, PathBuilder, PathSegment, QuadBezier};
pub use point::{PartialPoint, Point};
pub use poly::Polynomial;
pub use rectangle::{PhysicalRect, Rect};
pub use rounded_rectangle::RoundedRect;
use rustc_hash::FxBuildHasher;
pub use shape::Shape;
pub use size::{PhysicalSize, Size};
pub use text::*;
pub use transform::{Transform, TranslateScale};
pub use unit_point::{UnitPoint, UnitValue};
pub use vector::{Vec2, Vec2i, Vec2u, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowTheme {
    /// Light mode
    Light,
    /// Dark mode
    Dark,
}

pub trait Zero {
    const ZERO: Self;
}

impl Zero for f32 {
    const ZERO: Self = 0.0;
}

impl Zero for f64 {
    const ZERO: Self = 0.0;
}

/// Strong type for logical coordinates
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalCoord(pub i32);

impl Zero for PhysicalCoord {
    const ZERO: Self = Self(0);
}

impl Add for PhysicalCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for PhysicalCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScaleFactor(pub f32);

impl ScaleFactor {
    pub const fn logical_to_physical(&self) -> f32 {
        self.0
    }

    pub const fn physical_to_logical(&self) -> f32 {
        1.0 / self.0
    }
}

impl Display for ScaleFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Mul<f32> for ScaleFactor {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for ScaleFactor {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

pub(crate) type FxHashSet<K> = HashSet<K, FxBuildHasher>;
pub(crate) type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub(crate) type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;
pub(crate) type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
