use std::ops::Mul;

use super::{Point, Size, Vec2};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transform {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Transform {
    pub fn new(m11: f32, m12: f32, m21: f32, m22: f32, tx: f32, ty: f32) -> Self {
        Self {
            m11,
            m12,
            m21,
            m22,
            tx,
            ty,
        }
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    pub fn rotate(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self::new(c, -s, s, c, 0.0, 0.0)
    }

    pub fn translate(v: Vec2) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, v.x, v.y)
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self::new(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        Self {
            m11: self.m11 * rhs.m11 + self.m12 * rhs.m21,
            m12: self.m11 * rhs.m12 + self.m12 * rhs.m22,
            m21: self.m21 * rhs.m11 + self.m22 * rhs.m12,
            m22: self.m21 * rhs.m12 + self.m22 * rhs.m22,
            tx: self.tx + self.m11 * rhs.tx + self.m12 * rhs.ty,
            ty: self.ty + self.m21 * rhs.tx + self.m22 * rhs.ty,
        }
    }
}

impl Mul<Vec2> for Transform {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2::new(
            self.m11 * rhs.x + self.m12 * rhs.y + self.tx,
            self.m21 * rhs.x + self.m22 * rhs.y + self.ty,
        )
    }
}

impl Mul<Point> for Transform {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point::new(
            self.m11 * rhs.x + self.m12 * rhs.y + self.tx,
            self.m21 * rhs.x + self.m22 * rhs.y + self.ty,
        )
    }
}

impl Mul<Size> for Transform {
    type Output = Size;

    fn mul(self, rhs: Size) -> Self::Output {
        Size::new(
            self.m11 * rhs.width + self.m12 * rhs.height,
            self.m21 * rhs.width + self.m22 * rhs.height,
        )
    }
}

/// Transform consisting of a uniform scaling followed by a translation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslateScale {
    pub translation: Vec2,
    pub scale: f32,
}

impl TranslateScale {
    pub const fn new(translation: Vec2, scale: f32) -> Self {
        Self { translation, scale }
    }

    pub const fn identity() -> Self {
        Self::new(Vec2::ZERO, 1.0)
    }

    pub const fn translate(translation: Vec2) -> Self {
        Self::new(translation, 0.0)
    }

    pub const fn scale(scale: f32) -> Self {
        Self::new(Vec2::ZERO, scale)
    }

    pub fn inverse(self) -> Self {
        Self::new(-self.translation / self.scale, 1.0 / self.scale)
    }
}

impl Mul<Self> for TranslateScale {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            translation: self.translation + self.scale * rhs.translation,
            scale: self.scale * rhs.scale,
        }
    }
}

impl Mul<Vec2> for TranslateScale {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self.scale
    }
}

impl Mul<Point> for TranslateScale {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        rhs.scale(self.scale) + self.translation
    }
}

impl Mul<Size> for TranslateScale {
    type Output = Size;

    fn mul(self, rhs: Size) -> Self::Output {
        rhs * self.scale
    }
}

impl From<TranslateScale> for Transform {
    fn from(value: TranslateScale) -> Self {
        Self {
            m11: value.scale,
            m12: 0.0,
            m21: 0.0,
            m22: value.scale,
            tx: value.translation.x,
            ty: value.translation.y,
        }
    }
}
