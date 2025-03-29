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

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
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

impl Interpolate for Vector {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            x: self.x.lerp(&other.x, scalar),
            y: self.y.lerp(&other.y, scalar),
        }
    }
}
