use super::{Point, Rectangle, Size, Vec2};
use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RoundedRectangle<T = f64> {
    pub rect: Rectangle<T>,
    pub corner_radius: Size<T>,
}

impl<T> RoundedRectangle<T> {
    pub fn new(rect: Rectangle<T>, corner_radius: Size<T>) -> Self {
        Self {
            rect,
            corner_radius,
        }
    }
}

impl<T> RoundedRectangle<T>
where
    T: Debug + Copy + PartialEq + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + PartialOrd,
{
    pub fn contains(&self, pos: Point<T>) -> bool {
        if !self.rect.contains(pos) {
            false
        } else {
            // Check corners...
            true
        }
    }

    pub fn bounds(&self) -> Rectangle<T> {
        self.rect
    }
}

impl RoundedRectangle<f64> {
    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        Self::new(self.rect.offset(delta), self.corner_radius)
    }

    pub fn shrink(&self, amount: f64) -> Self {
        Self::new(
            self.rect.shrink(amount),
            self.corner_radius - Size::new(amount, amount),
        )
    }

    pub fn scale(&self, scale: f64) -> Self {
        Self::new(self.rect.scale(scale), self.corner_radius.scale(scale))
    }

    pub fn scale_x(&self, scale: f64) -> Self {
        Self::new(self.rect.scale_x(scale), self.corner_radius.scale_x(scale))
    }

    pub fn scale_y(&self, scale: f64) -> Self {
        Self::new(self.rect.scale_y(scale), self.corner_radius.scale_y(scale))
    }
}
