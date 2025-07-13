use std::fmt::Debug;
use std::ops::{Add, Mul, Neg, Sub};

use super::Size;
use super::Vec2;
use super::{Interpolate, Point};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rectangle<T = f64> {
    pub pos: Point<T>,
    pub size: Size<T>,
}

impl<T> Rectangle<T> {
    pub const fn from_origin(point: Point<T>, size: Size<T>) -> Self {
        Self { pos: point, size }
    }

    pub const fn from_xywh(x: T, y: T, width: T, height: T) -> Self {
        Self {
            pos: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[inline]
    pub fn from_ltrb(left: T, top: T, right: T, bottom: T) -> Self
    where
        T: PartialOrd + Sub<Output = T> + Copy,
    {
        assert!(left <= right);
        assert!(top <= bottom);
        Self {
            pos: Point::new(left, top),
            size: Size::new(right - left, bottom - top),
        }
    }

    #[inline]
    pub fn from_points(x0: Point<T>, x1: Point<T>) -> Self
    where
        T: PartialOrd + Sub<Output = T> + Copy,
    {
        let (x_min, x_max) = if x0.x < x1.x {
            (x0.x, x1.x)
        } else {
            (x1.x, x0.x)
        };
        let (y_min, y_max) = if x0.y < x1.y {
            (x0.y, x1.y)
        } else {
            (x1.y, x0.y)
        };
        Self {
            pos: Point::new(x_min, y_min),
            size: Size::new(x_max - x_min, y_max - y_min),
        }
    }

    pub fn with_origin(self, position: Point<T>) -> Self {
        Self {
            pos: position,
            size: self.size,
        }
    }

    pub fn with_size(self, size: Size<T>) -> Self {
        Self {
            pos: self.pos,
            size,
        }
    }
}

impl<T> Rectangle<T>
where
    T: Copy
        + PartialEq
        + Debug
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Neg<Output = T>
        + PartialOrd,
{
    #[inline]
    pub fn left(&self) -> T {
        self.pos.x
    }

    #[inline]
    pub fn right(&self) -> T {
        self.pos.x + self.size.width
    }

    #[inline]
    pub fn top(&self) -> T {
        self.pos.y
    }

    #[inline]
    pub fn bottom(&self) -> T {
        self.pos.y + self.size.height
    }

    #[inline]
    pub fn bottom_left(&self) -> Point<T> {
        Point::new(self.left(), self.bottom())
    }

    #[inline]
    pub fn top_left(&self) -> Point<T> {
        self.pos
    }

    #[inline]
    pub fn bottom_right(&self) -> Point<T> {
        Point::new(self.right(), self.bottom())
    }

    #[inline]
    pub fn top_right(&self) -> Point<T> {
        Point::new(self.right(), self.top())
    }

    pub fn get_relative_point(&self, rel_x: T, rel_y: T) -> Point<T> {
        Point::new(
            self.pos.x + rel_x * self.size.width,
            self.pos.y + rel_y * self.size.height,
        )
    }

    pub const fn origin(&self) -> Point<T> {
        self.pos
    }

    pub const fn size(&self) -> Size<T> {
        self.size
    }

    pub fn width(&self) -> T {
        self.size.width
    }

    pub fn height(&self) -> T {
        self.size.height
    }

    pub fn contains(&self, point: Point<T>) -> bool {
        point.x >= self.pos.x
            && (point.x - self.size.width) <= self.pos.x
            && point.y >= self.pos.y
            && (point.y - self.size.height) <= self.pos.y
    }

    pub fn intersects(&self, other: &Self) -> bool {
        !(self.left() > other.right()
            || self.right() < other.left()
            || self.top() > other.bottom()
            || self.bottom() < other.top())
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
            pos: self.pos + Point::splat(amount),
            size: self.size - Size::splat(amount + amount),
        }
    }

    /// Shrink the rectangle by `amount` in the x direction, keeping the same center position
    pub fn shrink_x(&self, amount: T) -> Self {
        Self {
            pos: Point::new(self.pos.x + amount, self.pos.y),
            size: Size::new(self.size.width - (amount + amount), self.size.height),
        }
    }

    /// Shrink the rectangle by `amount` in the y direction, keeping the same center position
    pub fn shrink_y(&self, amount: T) -> Self {
        Self {
            pos: Point::new(self.pos.x, self.pos.y + amount),
            size: Size::new(self.size.width, self.size.height - (amount + amount)),
        }
    }
}

impl Rectangle<f64> {
    pub const EMPTY: Self = Self {
        pos: Point::ZERO,
        size: Size::ZERO,
    };

    pub fn from_center(center: Point, size: Size) -> Self {
        Self {
            pos: center - size / 2.0,
            size,
        }
    }

    pub fn with_center(self, center: Point) -> Self {
        Self::from_center(center, self.size)
    }

    pub fn with_size_keeping_center(self, size: Size) -> Self {
        Self {
            pos: self.pos - (size - self.size) / 2.0,
            size,
        }
    }

    pub fn center(&self) -> Point {
        self.pos + (self.size / 2.0)
    }

    pub fn scale(&self, scale: f64) -> Self {
        Self::from_origin(self.origin().scale(scale), self.size().scale(scale))
    }

    pub fn scale_x(&self, scale: f64) -> Self {
        Self::from_origin(self.origin().scale_x(scale), self.size().scale_x(scale))
    }

    pub fn scale_y(&self, scale: f64) -> Self {
        Self::from_origin(self.origin().scale_y(scale), self.size().scale_y(scale))
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        Self::from_origin(self.origin() + delta.into(), self.size())
    }

    pub fn combine_with(&self, other: &Self) -> Self {
        let left = self.left().min(other.left());
        let right = self.right().max(other.right());
        let top = self.top().min(other.top());
        let bottom = self.bottom().max(other.bottom());
        Self::from_ltrb(left, top, right, bottom)
    }
}

impl From<Rectangle<i32>> for Rectangle<f64> {
    fn from(value: Rectangle<i32>) -> Self {
        Self {
            pos: value.pos.into(),
            size: value.size.into(),
        }
    }
}

impl<T: Default> Default for Rectangle<T> {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            size: Default::default(),
        }
    }
}

impl<T: Interpolate> Interpolate for Rectangle<T> {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            pos: self.pos.lerp(&other.pos, scalar),
            size: self.size.lerp(&other.size, scalar),
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
        let rect = Rectangle::from_points(p0, p1);

        assert_eq!(rect.left(), 1.0);
        assert_eq!(rect.top(), 2.0);
        assert_eq!(rect.right(), 3.0);
        assert_eq!(rect.bottom(), 4.0);
    }

    #[test]
    fn contains() {
        let p0 = Point::new(1, 22);
        let p1 = Point::new(16, 2);
        let rect = Rectangle::from_points(p0, p1);

        assert!(rect.contains(p0));
        assert!(rect.contains(p1));
        assert!(rect.contains(Point::new(10, 18)));

        assert!(!rect.contains(Point::new(1, 23)));
        assert!(!rect.contains(Point::new(1, 1)));

        assert!(!rect.contains(Point::new(0, 4)));
        assert!(!rect.contains(Point::new(17, 4)));
    }
}
