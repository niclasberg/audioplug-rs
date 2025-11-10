use std::fmt::Debug;
use std::ops::{Add, Mul, Neg, Sub};

use bytemuck::{Pod, Zeroable};

use crate::core::{PhysicalCoord, RoundedRect, ScaleFactor};

use super::Point;
use super::Size;
use super::Vec2;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct Rect<T = f64> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Rect<T> {
    #[inline]
    pub fn from_points(x0: Point<T>, x1: Point<T>) -> Self
    where
        T: PartialOrd + Sub<Output = T> + Copy,
    {
        let (left, right) = if x0.x < x1.x {
            (x0.x, x1.x)
        } else {
            (x1.x, x0.x)
        };
        let (top, bottom) = if x0.y < x1.y {
            (x0.y, x1.y)
        } else {
            (x1.y, x0.y)
        };
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn from_origin(point: Point<T>, size: Size<T>) -> Self
    where
        T: Copy + Add<Output = T>,
    {
        Self {
            left: point.x,
            top: point.y,
            right: point.x + size.width,
            bottom: point.y + size.height,
        }
    }

    pub fn from_xywh(left: T, top: T, width: T, height: T) -> Self
    where
        T: Copy + Add<Output = T>,
    {
        Self {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    pub fn with_origin(self, position: Point<T>) -> Self
    where
        T: Copy + Add<Output = T> + Sub<Output = T>,
    {
        Self {
            left: position.x,
            top: position.y,
            right: position.x + (self.right - self.left),
            bottom: position.y,
        }
    }

    pub fn with_size(self, size: Size<T>) -> Self
    where
        T: Copy + Add<Output = T>,
    {
        Self {
            left: self.left,
            top: self.top,
            right: self.left + size.width,
            bottom: self.top + size.height,
        }
    }

    #[inline]
    pub const fn bottom_left(self) -> Point<T>
    where
        T: Copy,
    {
        Point::new(self.left, self.bottom)
    }

    #[inline]
    pub const fn top_left(self) -> Point<T>
    where
        T: Copy,
    {
        Point::new(self.left, self.top)
    }

    #[inline]
    pub const fn bottom_right(self) -> Point<T>
    where
        T: Copy,
    {
        Point::new(self.right, self.bottom)
    }

    #[inline]
    pub const fn top_right(self) -> Point<T>
    where
        T: Copy,
    {
        Point::new(self.right, self.top)
    }

    pub fn size(&self) -> Size<T>
    where
        T: Sub<Output = T> + Copy,
    {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    pub fn width(self) -> T
    where
        T: Sub<Output = T> + Copy,
    {
        self.right - self.left
    }

    pub fn height(self) -> T
    where
        T: Sub<Output = T> + Copy,
    {
        self.bottom - self.top
    }

    pub fn contains(&self, point: Point<T>) -> bool
    where
        T: PartialOrd,
    {
        point.x >= self.left
            && point.x <= self.right
            && point.y >= self.top
            && point.y <= self.bottom
    }

    pub fn intersects(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        !(self.left > other.right
            || self.right < other.left
            || self.top > other.bottom
            || self.bottom < other.top)
    }
}

impl<T> Rect<T>
where
    T: Copy
        + PartialEq
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Neg<Output = T>
        + PartialOrd,
{
    pub fn get_relative_point(&self, rel_x: T, rel_y: T) -> Point<T> {
        Point::new(
            self.left + rel_x * self.width(),
            self.top + rel_y * self.height(),
        )
    }

    /// Expand the rectangle by `amount` from each side. Retains the
    /// center position and reduces the size by 2 times `amount`
    pub fn expand(&self, amount: T) -> Self {
        Self::shrink(self, -amount)
    }

    /// Expand the rectangle by `amount` in the x direction, keeping the same center position
    pub fn expand_x(&self, amount: T) -> Self {
        Self::shrink_x(self, -amount)
    }

    /// Expand the rectangle by `amount` in the y direction, keeping the same center position
    pub fn expand_y(&self, amount: T) -> Self {
        Self::shrink_y(self, -amount)
    }

    /// Shrink the rectangle by `amount` from each side. Retains the
    /// center position and reduces the size by 2 times `amount`
    pub fn shrink(&self, amount: T) -> Self {
        Self {
            left: self.left + amount,
            top: self.top + amount,
            right: self.right - amount,
            bottom: self.bottom - amount,
        }
    }

    /// Shrink the rectangle by `amount` in the x direction, keeping the same center position
    pub fn shrink_x(&self, amount: T) -> Self {
        Self {
            left: self.left + amount,
            top: self.top,
            right: self.right - amount,
            bottom: self.bottom,
        }
    }

    /// Shrink the rectangle by `amount` in the y direction, keeping the same center position
    pub fn shrink_y(&self, amount: T) -> Self {
        Self {
            left: self.left,
            top: self.top + amount,
            right: self.right,
            bottom: self.bottom - amount,
        }
    }
}

impl Rect<f64> {
    pub const EMPTY: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    pub fn from_center(center: Point, size: Size) -> Self {
        let half_width = size.width / 2.0;
        let half_height = size.height / 2.0;
        Self {
            left: center.x - half_width,
            top: center.y - half_height,
            right: center.x + half_width,
            bottom: center.y + half_width,
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

    pub fn center(&self) -> Point {
        Point::new(
            0.5 * (self.left + self.right),
            0.5 * (self.top + self.bottom),
        )
    }

    pub fn scale(&self, scale: f64) -> Self {
        Self {
            left: self.left * scale,
            top: self.top * scale,
            right: self.right * scale,
            bottom: self.bottom * scale,
        }
    }

    pub fn scale_x(&self, scale: f64) -> Self {
        Self {
            left: self.left * scale,
            top: self.top,
            right: self.right * scale,
            bottom: self.bottom,
        }
    }

    pub fn scale_y(&self, scale: f64) -> Self {
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

    pub fn into_rounded_rect(self, corner_radius: Size) -> RoundedRect {
        RoundedRect::new(self, corner_radius)
    }
}

impl From<Rect<i32>> for Rect<f64> {
    fn from(value: Rect<i32>) -> Self {
        Self {
            left: value.left as _,
            top: value.top as _,
            right: value.right as _,
            bottom: value.bottom as _,
        }
    }
}

impl From<Rect<f64>> for Rect<f32> {
    fn from(value: Rect<f64>) -> Self {
        Self {
            left: value.left as _,
            top: value.top as _,
            right: value.right as _,
            bottom: value.bottom as _,
        }
    }
}

unsafe impl Zeroable for Rect<f32> {}
unsafe impl Pod for Rect<f32> {}

/*impl<T: Interpolate> Interpolate for Rect<T> {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            pos: self.pos.lerp(&other.pos, scalar),
            size: self.size.lerp(&other.size, scalar),
        }
    }
}*/

pub type PhysicalRect = Rect<PhysicalCoord>;
impl PhysicalRect {
    pub fn from_logical(rect: Rect, scale_factor: ScaleFactor) -> Self {
        Self {
            left: PhysicalCoord((rect.left * scale_factor.0).floor() as i32),
            top: PhysicalCoord((rect.top * scale_factor.0).floor() as i32),
            right: PhysicalCoord((rect.right * scale_factor.0).ceil() as i32),
            bottom: PhysicalCoord((rect.bottom * scale_factor.0).ceil() as i32),
        }
    }

    pub fn into_logical(self, scale_factor: ScaleFactor) -> Rect {
        Rect::from_origin(
            self.top_left().into_logical(scale_factor),
            self.size().into_logical(scale_factor),
        )
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

    #[test]
    fn contains() {
        let p0 = Point::new(1, 22);
        let p1 = Point::new(16, 2);
        let rect = Rect::from_points(p0, p1);

        assert!(rect.contains(p0));
        assert!(rect.contains(p1));
        assert!(rect.contains(Point::new(10, 18)));

        assert!(!rect.contains(Point::new(1, 23)));
        assert!(!rect.contains(Point::new(1, 1)));

        assert!(!rect.contains(Point::new(0, 4)));
        assert!(!rect.contains(Point::new(17, 4)));
    }
}
