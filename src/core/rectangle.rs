use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};

use super::{Interpolate, Point};
use super::Size;
use super::Vector;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rectangle<T = f64> {
    pub pos: Point<T>,
    pub size: Size<T>,
}

impl<T> Rectangle<T> {
    pub const fn new(point: Point<T>, size: Size<T>) -> Self {
        Self { pos: point, size }
    }

    pub const fn from_xywh(x: T, y: T, width: T, height: T) -> Self {
        Self { pos: Point::new(x, y), size: Size::new(width, height) }
    }

    #[inline]
    pub fn from_ltrb(left: T, top: T, right: T, bottom: T) -> Self 
        where T: PartialOrd + Sub<Output=T> + Copy 
    {
        assert!(left <= right);
        assert!(top <= bottom);
        Self { pos: Point::new(left, top), size: Size::new(right - left, bottom - top)}
    }

    #[inline]
    pub fn from_points(x0: Point<T>, x1: Point<T>) -> Self 
        where T: PartialOrd + Sub<Output=T> + Copy 
    {
        let (x_min, x_max) = if x0.x < x1.x { (x0.x, x1.x) } else { (x1.x, x0.x) };
        let (y_min, y_max) = if x0.y < x1.y { (x0.y, x1.y) } else { (x1.y, x0.y) };
        Self { pos: Point::new(x_min, x_max), size: Size::new(x_max - x_min, y_max - y_min) }
    }
}

impl<T> Rectangle<T> 
where T: Copy + PartialEq + Debug + Add<Output = T> + Sub<Output=T> + Mul<Output=T> + PartialOrd
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
        Point::new(self.left(), self.top())
    }

    #[inline]
    pub fn bottom_right(&self) -> Point<T> {
        Point::new(self.right(), self.bottom())
    }

    #[inline]
    pub fn top_right(&self) -> Point<T> {
        Point::new(self.right(), self.top())
    }

    pub const fn position(&self) -> Point<T> {
        self.pos
    }

    pub fn with_position(&self, position: Point<T>) -> Self {
        Self::new(position, self.size())
    }

    pub const fn size(&self) -> Size<T> {
        self.size
    }

    pub fn with_size(&self, size: Size<T>) -> Self {
        Self::new(self.position(), size)
    }

    pub fn width(&self) -> T {
        self.size.width
    }

    pub fn height(&self) -> T {
        self.size.height
    }

    pub fn contains(&self, point: Point<T>) -> bool {
        point.x >= self.pos.x && (point.x - self.size.width) <= self.pos.x &&
        point.y >= self.pos.y && (point.y - self.size.height) <= self.pos.y
    }

    pub fn intersects(&self, other: &Self) -> bool {
        !(
            self.left() > other.right() ||
            self.right() < other.left() ||
            self.top() < other.bottom() ||
            self.bottom() > other.bottom()
        )
    }
}

impl Rectangle<f64> {
    pub fn from_center(center: Point, size: Size) -> Self {
        Self::new(center - size / 2.0, size)
    }

    /// Shrink the rectangle by `amount`, keeping the same center position
    pub fn shrink(&self, amount: f64) -> Self {
        Self::from_center(self.center(), self.size() - Size::new(amount, amount))
    }

    pub fn shrink_x(&self, amount: f64) -> Self {
        Self::from_center(self.center(), self.size() - Size::new(amount, 0.0))
    }

    pub fn shrink_y(&self, amount: f64) -> Self {
        Self::from_center(self.center(), self.size() - Size::new(0.0, amount))
    }

    pub fn center(&self) -> Point {
        self.pos + (self.size / 2.0)
    }

    pub fn scale(&self, scale: f64) -> Self{
        Self::new(self.position().scale(scale), self.size().scale(scale))
    }

    pub fn scale_x(&self, scale: f64) -> Self{
        Self::new(self.position(), self.size().scale_x(scale))
    }

    pub fn scale_y(&self, scale: f64) -> Self{
        Self::new(self.position(), self.size().scale_y(scale))
    }

    pub fn offset(&self, delta: impl Into<Vector>) -> Self {
        Self::new(self.position()+delta.into(), self.size())
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
		Self { pos: Default::default(), size: Default::default() }
	}
}

impl<T: Interpolate> Interpolate for Rectangle<T> {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            pos: self.pos.lerp(&other.pos, scalar),
            size: self.size.lerp(&other.size, scalar)
        }
    }
}

#[cfg(test)]
mod test {
    /*use super::*;

    #[test]
    fn from_points() {
        let p0 = Point::new(1.0, 4.0);
        let p1 = Point::new(3.0, 2.0);
        let rect = Rectangle::from_points(p0, p1);

        assert_eq!(rect.left(), p0.x);
        assert_eq!(rect.top(), p0.y);
        assert_eq!(rect.right(), p1.x);
        assert_eq!(rect.bottom(), p1.y);
    }

    #[test]
    fn contains() {
        let p0 = Point::new(1, 22);
        let p1 = Point::new(16, 2);
        let rect = Rectangle::from_points(p0, p1);
        println!("{:?}", rect);
        assert!(rect.contains(p0));
        assert!(rect.contains(p1));
        assert!(rect.contains(Point::new(10, 18)));
        
        assert!(!rect.contains(Point::new(1, 23)));
        assert!(!rect.contains(Point::new(1, 1)));
        
        assert!(!rect.contains(Point::new(0, 4)));
        assert!(!rect.contains(Point::new(17, 4)));
    }*/
}