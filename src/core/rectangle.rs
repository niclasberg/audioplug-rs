use std::fmt::Debug;
use std::ops::Mul;

use bytemuck::{Pod, Zeroable};

use crate::core::{RoundedRect, ScaleFactor};
use crate::core::{TranslateScale, Zero};

use super::Point;
use super::Size;
use super::Vec2;

#[repr(C)]
#[derive(Debug, Default, PartialEq, Clone, Copy, Pod, Zeroable)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub const EMPTY: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    #[inline]
    pub fn from_points(x0: Point, x1: Point) -> Self {
        let min = x0.min(x1);
        let max = x0.max(x1);
        Self {
            left: min.x,
            top: min.y,
            right: max.x,
            bottom: max.y,
        }
    }

    pub fn from_origin(point: Point, size: Size) -> Self {
        Self {
            left: point.x,
            top: point.y,
            right: point.x + size.width,
            bottom: point.y + size.height,
        }
    }

    pub fn from_xywh(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    pub fn from_union(rects: &[Self]) -> Self {
        let mut it = rects.iter();
        if let Some(first) = it.next().copied() {
            it.fold(first, |res, rect| res.union(rect))
        } else {
            Self::EMPTY
        }
    }

    pub fn with_origin(self, position: Point) -> Self {
        Self {
            left: position.x,
            top: position.y,
            right: position.x + (self.right - self.left),
            bottom: position.y,
        }
    }

    pub fn with_size(self, size: Size) -> Self {
        Self {
            left: self.left,
            top: self.top,
            right: self.left + size.width,
            bottom: self.top + size.height,
        }
    }

    #[inline]
    pub const fn bottom_left(self) -> Point {
        Point::new(self.left, self.bottom)
    }

    #[inline]
    pub const fn top_left(self) -> Point {
        Point::new(self.left, self.top)
    }

    #[inline]
    pub const fn bottom_right(self) -> Point {
        Point::new(self.right, self.bottom)
    }

    #[inline]
    pub const fn top_right(self) -> Point {
        Point::new(self.right, self.top)
    }

    pub const fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    pub const fn width(self) -> f32 {
        self.right - self.left
    }

    pub const fn height(self) -> f32 {
        self.bottom - self.top
    }

    pub const fn contains(&self, point: Point) -> bool {
        point.x >= self.left
            && point.x <= self.right
            && point.y >= self.top
            && point.y <= self.bottom
    }

    pub fn intersects(&self, other: &Self) -> bool {
        !(self.left > other.right
            || self.right < other.left
            || self.top > other.bottom
            || self.bottom < other.top)
    }

    pub fn get_relative_point(&self, rel_x: f32, rel_y: f32) -> Point {
        Point::new(
            self.left + rel_x * self.width(),
            self.top + rel_y * self.height(),
        )
    }

    /// Expand the rectangle by `amount` from each side. Retains the
    /// center position and reduces the size by 2 times `amount`
    pub fn inflate(&self, amount: f32) -> Self {
        Self::shrink(self, -amount)
    }

    /// Expand the rectangle by `amount` in the x direction, keeping the same center position
    pub fn inflate_x(&self, amount: f32) -> Self {
        Self::shrink_x(self, -amount)
    }

    /// Expand the rectangle by `amount` in the y direction, keeping the same center position
    pub fn inflate_y(&self, amount: f32) -> Self {
        Self::shrink_y(self, -amount)
    }

    /// Shrink the rectangle by `amount` from each side. Retains the
    /// center position and reduces the size by 2 times `amount`
    pub fn shrink(&self, amount: f32) -> Self {
        Self {
            left: self.left + amount,
            top: self.top + amount,
            right: self.right - amount,
            bottom: self.bottom - amount,
        }
    }

    /// Shrink the rectangle by `amount` in the x direction, keeping the same center position
    pub fn shrink_x(&self, amount: f32) -> Self {
        Self {
            left: self.left + amount,
            top: self.top,
            right: self.right - amount,
            bottom: self.bottom,
        }
    }

    /// Shrink the rectangle by `amount` in the y direction, keeping the same center position
    pub fn shrink_y(&self, amount: f32) -> Self {
        Self {
            left: self.left,
            top: self.top + amount,
            right: self.right,
            bottom: self.bottom - amount,
        }
    }

    pub const fn from_center(center: Point, size: Size) -> Self {
        let half_width = size.width / 2.0;
        let half_height = size.height / 2.0;
        Self {
            left: center.x - half_width,
            top: center.y - half_height,
            right: center.x + half_width,
            bottom: center.y + half_height,
        }
    }

    pub fn with_center(self, center: Point) -> Self {
        Self::from_center(center, self.size())
    }

    pub fn with_size_keeping_center(self, size: Size) -> Self {
        let size_diff = (size - self.size()) / 2.0;
        Self {
            left: self.left - size_diff.width,
            top: self.top - size_diff.height,
            right: self.right + size_diff.width,
            bottom: self.bottom + size_diff.height,
        }
    }

    pub const fn center(&self) -> Point {
        Point::new(
            0.5 * (self.left + self.right),
            0.5 * (self.top + self.bottom),
        )
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self {
            left: self.left * scale,
            top: self.top * scale,
            right: self.right * scale,
            bottom: self.bottom * scale,
        }
    }

    pub fn scale_x(&self, scale: f32) -> Self {
        Self {
            left: self.left * scale,
            top: self.top,
            right: self.right * scale,
            bottom: self.bottom,
        }
    }

    pub fn scale_y(&self, scale: f32) -> Self {
        Self {
            left: self.left,
            top: self.top * scale,
            right: self.right,
            bottom: self.bottom * scale,
        }
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        let delta: Vec2 = delta.into();
        Self {
            left: self.left + delta.x,
            top: self.top + delta.y,
            right: self.right + delta.x,
            bottom: self.bottom + delta.y,
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        match (self.size() == Size::ZERO, other.size() == Size::ZERO) {
            (true, true) => Self::EMPTY,
            (true, false) => *other,
            (false, true) => *self,
            (false, false) => {
                let left = self.left.min(other.left);
                let right = self.right.max(other.right);
                let top = self.top.min(other.top);
                let bottom = self.bottom.max(other.bottom);
                Self {
                    left,
                    top,
                    right,
                    bottom,
                }
            }
        }
    }

    pub fn expand_to_include(&self, point: Point) -> Self {
        Self {
            left: self.left.min(point.x),
            top: self.top.min(point.y),
            right: self.right.max(point.x),
            bottom: self.bottom.max(point.y),
        }
    }

    pub fn into_rounded_rect(self, corner_radius: Size) -> RoundedRect {
        RoundedRect::new(self, corner_radius)
    }
}

impl Mul<Rect> for TranslateScale {
    type Output = Rect;

    fn mul(self, rhs: Rect) -> Self::Output {
        Rect::from_points(self * rhs.top_left(), self * rhs.bottom_right())
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct PhysicalRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl PhysicalRect {
    pub fn from_logical(rect: Rect, scale_factor: ScaleFactor) -> Self {
        Self {
            left: (rect.left * scale_factor.0).floor() as i32,
            top: (rect.top * scale_factor.0).floor() as i32,
            right: (rect.right * scale_factor.0).ceil() as i32,
            bottom: (rect.bottom * scale_factor.0).ceil() as i32,
        }
    }

    pub fn into_logical(self, scale_factor: ScaleFactor) -> Rect {
        Rect {
            left: self.left as f32 / scale_factor.0,
            top: self.top as f32 / scale_factor.0,
            right: self.right as f32 / scale_factor.0,
            bottom: self.bottom as f32 / scale_factor.0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_points() {
        let p0 = Point::new(1.0, 4.0);
        let p1 = Point::new(3.0, 2.0);
        let rect = Rect::from_points(p0, p1);

        assert_eq!(rect.left, 1.0);
        assert_eq!(rect.top, 2.0);
        assert_eq!(rect.right, 3.0);
        assert_eq!(rect.bottom, 4.0);
    }
}
