use std::ops::Mul;

use super::{Vector, Point, Size};

pub struct Transform<T = f64> {
    pub m11: T,
	pub m12: T, 
	pub m21: T,
	pub m22: T,
	pub tx: T,
	pub ty: T
}

impl<T> Transform<T> {
	pub fn new(m11: T, m12: T, m21: T, m22: T, tx: T, ty: T) -> Self {
		Self { m11, m12, m21, m22, tx, ty }
	}
}

impl Transform {
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    pub fn translate(v: Vector) -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, v.x, v.y)
    }

	pub fn scale(sx: f64, sy: f64) -> Self {
		Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
	}
}

impl Mul<Vector> for Transform {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector::new(
			self.m11 * rhs.x + self.m12 * rhs.y + self.tx, 
			self.m21 * rhs.x + self.m22 * rhs.y + self.ty
		)
    }
}

impl Mul<Point> for Transform {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point::new(
			self.m11 * rhs.x + self.m12 * rhs.y + self.tx, 
			self.m21 * rhs.x + self.m22 * rhs.y + self.ty
		)
    }
}

impl Mul<Size> for Transform {
    type Output = Size;

    fn mul(self, rhs: Size) -> Self::Output {
        Size::new(
			self.m11 * rhs.width + self.m12 * rhs.height, 
			self.m21 * rhs.width + self.m22 * rhs.height
		)
    }
}