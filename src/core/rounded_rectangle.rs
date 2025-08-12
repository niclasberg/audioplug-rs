use super::{Point, Rect, Size, Vec2};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RoundedRect {
    pub rect: Rect,
    pub corner_radius: Size,
}

impl RoundedRect {
    pub fn new(rect: Rect, corner_radius: Size) -> Self {
        Self {
            rect,
            corner_radius,
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        if !self.rect.contains(pos) {
            false
        } else {
            // Check corners...
            true
        }
    }

    pub fn bounds(&self) -> Rect {
        self.rect
    }

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
