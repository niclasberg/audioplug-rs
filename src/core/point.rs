use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use super::{interpolation::SpringPhysics, Interpolate, Size, Vector};

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

    pub fn as_vector(self) -> Vector {
        Vector::new(self.x, self.y)
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

impl<T: Add<Output = T>> Add for Point<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
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

impl Add<Vector> for Point {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
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

impl Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
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
