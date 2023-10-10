use super::{Point, Size};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

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