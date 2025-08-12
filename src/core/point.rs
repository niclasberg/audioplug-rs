use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use crate::core::{PhysicalCoord, ScaleFactor};

use super::{Interpolate, Size, SpringPhysics, Vec2};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point<T = f64> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Create a Point with both `x` and `y` set to `v`
    #[inline]
    #[must_use]
    pub fn splat(v: T) -> Self
    where
        T: Clone,
    {
        Self { x: v.clone(), y: v }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Point<U> {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }

    pub fn map_x(self, f: impl Fn(T) -> T) -> Self {
        Self {
            x: f(self.x),
            y: self.y,
        }
    }

    pub fn map_y(self, f: impl Fn(T) -> T) -> Self {
        Self {
            x: self.x,
            y: f(self.y),
        }
    }

    pub fn max(&self, other: &Self) -> Self
    where
        T: PartialOrd + Copy,
    {
        Self {
            x: if self.x > other.x { self.x } else { other.x },
            y: if self.y > other.y { self.y } else { other.y },
        }
    }

    pub fn min(&self, other: &Self) -> Self
    where
        T: PartialOrd + Copy,
    {
        Self {
            x: if self.x < other.x { self.x } else { other.x },
            y: if self.y < other.y { self.y } else { other.y },
        }
    }
}

impl<T> Point<Option<T>> {
    pub fn unwrap_or(self, other: Point<T>) -> Point<T> {
        Point {
            x: self.x.unwrap_or(other.x),
            y: self.y.unwrap_or(other.y),
        }
    }
}

impl Point<f64> {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn zero() -> Self {
        Self { x: 0f64, y: 0f64 }
    }

    pub fn scale(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s)
    }

    pub fn scale_x(self, s: f64) -> Self {
        Self::new(self.x * s, self.y)
    }

    pub fn scale_y(self, s: f64) -> Self {
        Self::new(self.x, self.y * s)
    }

    pub fn max_element(&self) -> f64 {
        self.x.max(self.y)
    }

    pub fn min_element(&self) -> f64 {
        self.x.min(self.y)
    }

    #[inline(always)]
    pub fn into_vector(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
}

pub type PhysicalPoint = Point<PhysicalCoord>;
impl PhysicalPoint {
    pub fn into_logical(self, scale_factor: ScaleFactor) -> Point {
        Point::new(
            self.x.0 as f64 * scale_factor.0,
            self.y.0 as f64 * scale_factor.0,
        )
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}))", self.x, self.y)
    }
}

impl From<Point<i32>> for Point<f64> {
    fn from(value: Point<i32>) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl<T, U: Into<T>> From<[U; 2]> for Point<T> {
    fn from([x, y]: [U; 2]) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<T, U: Into<T>> From<(U, U)> for Point<T> {
    fn from((x, y): (U, U)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<T: Add<Output = T>> Add<Size<T>> for Point<T> {
    type Output = Self;

    fn add(self, rhs: Size<T>) -> Self::Output {
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

impl<T: Sub<Output = T>> Sub for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Sub<Output = T>> Sub<Size<T>> for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Size<T>) -> Self::Output {
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

impl<T: Default> Default for Point<T> {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }
}

impl<T: Interpolate> Interpolate for Point<T> {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            x: self.x.lerp(&other.x, scalar),
            y: self.y.lerp(&other.y, scalar),
        }
    }
}

impl<T: SpringPhysics> SpringPhysics for Point<T> {
    const ZERO: Self = Point::new(T::ZERO, T::ZERO);

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
