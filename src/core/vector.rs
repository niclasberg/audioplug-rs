use std::ops::{Add, Sub};

use super::{Interpolate, Point, Size};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn splat(val: f64) -> Self {
        Self { x: val, y: val }
    }

    pub const fn into_point(self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub const fn into_size(self) -> Size {
        Size {
            width: self.x,
            height: self.y,
        }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        Vector::new(value.x, value.y)
    }
}

impl From<Size> for Vector {
    fn from(value: Size) -> Self {
        Vector::new(value.width, value.height)
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Interpolate for Vector {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            x: self.x.lerp(&other.x, scalar),
            y: self.y.lerp(&other.y, scalar),
        }
    }
}
