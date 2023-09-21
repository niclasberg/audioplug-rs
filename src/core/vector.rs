use super::{Point, Size};

pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
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