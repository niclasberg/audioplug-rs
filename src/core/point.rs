use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::core::{ScaleFactor, Zero};

use super::{Lerp, Size, SpringPhysics, Vec2};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a Point with both `x` and `y` set to `v`
    #[inline]
    #[must_use]
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    pub fn map(self, f: impl Fn(f32) -> f32) -> Point {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }

    pub fn map_x(self, f: impl Fn(f32) -> f32) -> Self {
        Self {
            x: f(self.x),
            y: self.y,
        }
    }

    pub fn map_y(self, f: impl Fn(f32) -> f32) -> Self {
        Self {
            x: self.x,
            y: f(self.y),
        }
    }

    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: if self.x > other.x { self.x } else { other.x },
            y: if self.y > other.y { self.y } else { other.y },
        }
    }

    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max_element(self) -> f32 {
        self.x.max(self.y)
    }

    pub fn min_element(&self) -> f32 {
        self.x.min(self.y)
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn scale(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s)
    }

    pub fn scale_x(self, s: f32) -> Self {
        Self::new(self.x * s, self.y)
    }

    pub fn scale_y(self, s: f32) -> Self {
        Self::new(self.x, self.y * s)
    }

    #[inline(always)]
    pub fn into_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
}

impl Zero for Point {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}))", self.x, self.y)
    }
}

impl<U: Into<f32>> From<[U; 2]> for Point {
    fn from([x, y]: [U; 2]) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<U: Into<f32>> From<(U, U)> for Point {
    fn from((x, y): (U, U)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl Add<Size> for Point {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            x: self.x + rhs.width,
            y: self.y + rhs.height,
        }
    }
}

impl Add<Vec2> for Point {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Vec2> for Point {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Size> for Point {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        Self {
            x: self.x - rhs.width,
            y: self.y - rhs.height,
        }
    }
}

impl Sub<Vec2> for Point {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign<Vec2> for Point {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Lerp for Point {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            x: self.x.lerp(&other.x, scalar),
            y: self.y.lerp(&other.y, scalar),
        }
    }
}

impl SpringPhysics for Point {
    fn distance_squared_to(&self, other: &Self) -> f64 {
        self.x.distance_squared_to(&other.x) + self.y.distance_squared_to(&other.y)
    }

    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &super::SpringProperties,
    ) {
        self.x
            .apply_spring_update(&mut velocity.x, delta_t, &target.x, properties);
        self.y
            .apply_spring_update(&mut velocity.y, delta_t, &target.y, properties);
    }
}

#[derive(Default)]
pub struct PartialPoint {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

impl PartialPoint {
    pub fn empty() -> Self {
        Self { x: None, y: None }
    }

    pub fn splat(x: Option<f32>) -> Self {
        Self { x: x, y: x }
    }

    pub fn unwrap_or(self, default: Point) -> Point {
        Point {
            x: self.x.unwrap_or(default.x),
            y: self.y.unwrap_or(default.y),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

impl PhysicalPoint {
    pub fn into_logical(self, scale_factor: ScaleFactor) -> Point {
        Point::new(
            self.x as f32 * scale_factor.0,
            self.y as f32 * scale_factor.0,
        )
    }
}
