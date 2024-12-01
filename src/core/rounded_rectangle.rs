use std::ops::{Add, Mul, Sub};
use std::fmt::Debug;
use super::{Point, Rectangle, Size, Vector};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RoundedRectangle<T = f64> {
    pub rect: Rectangle<T>,
    pub corner_radius: Size<T>,
}

impl<T> RoundedRectangle<T> {
    pub fn new(rect: Rectangle<T>, corner_radius: Size<T>) -> Self {
        Self {
            rect, 
            corner_radius
        }
    }
}

impl<T> RoundedRectangle<T> 
where T: Debug + Copy + PartialEq + Add<Output = T> + Sub<Output=T> + Mul<Output=T> + PartialOrd
{
    pub fn contains(&self, pos: Point<T>) -> bool {
        if !self.rect.contains(pos) {
            false
        } else {
            // Check corners...
            true
        }
    }
}

impl RoundedRectangle<f64> {
    pub fn offset(&self, delta: impl Into<Vector>) -> Self {
        Self::new(self.rect.offset(delta), self.corner_radius)
    }
}