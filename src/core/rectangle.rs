use std::fmt::Debug;
use std::ops::Add;
use std::ops::Sub;

use super::Point;
use super::Size;
use super::Vector;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rectangle<T = f64> {
    x: T,
    y: T,
    width: T,
    height: T,
}

impl<T> Rectangle<T> 
where T: Copy + PartialEq + Debug + Add<Output = T> + Sub<Output=T> + PartialOrd
{
    pub fn new(point: Point<T>, size: Size<T>) -> Self {
        Self { x: point.x, y: point.y, width: size.width, height: size.height }
    }

    pub fn from_points(x0: Point<T>, x1: Point<T>) -> Self {
        assert!(x0.x <= x1.x);
        assert!(x0.y >= x1.y);
        Self { x: x0.x, y: x0.y, width: x1.x - x0.x, height: x0.y - x1.y }
    }

    pub fn from_xywh(x: T, y: T, width: T, height: T) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_ltrb(left: T, top: T, right: T, bottom: T) -> Self {
        Self { x: left, y: top, width: right - left, height: bottom - top}
    }

    pub fn left(&self) -> T {
        self.x
    }

    pub fn right(&self) -> T {
        self.x + self.width
    }

    pub fn top(&self) -> T {
        self.y
    }

    pub fn bottom(&self) -> T {
        self.y + self.height
    }

    pub fn position(&self) -> Point<T> {
        Point::new(self.x, self.y)
    }

    pub fn size(&self) -> Size<T> {
        Size::new(self.width, self.height)
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }

    pub fn contains(&self, point: Point<T>) -> bool {
        point.x >= self.x && (point.x - self.width) <= self.x &&
        point.y >= self.y && (point.y - self.height) <= self.y
    }
}

impl Rectangle<f64> {
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn intersects(&self, other: Rectangle) -> bool {
        !(
            self.left() > other.right() ||
            self.right() < other.left() ||
            self.top() < other.bottom() ||
            self.bottom() > other.bottom()
        )
    }

    pub fn scale(&self, scale: f64) -> Self{
        Self::new(self.position(), self.size().scale(scale))
    }

    pub fn scale_x(&self, scale: f64) -> Self{
        Self::new(self.position(), self.size().scale_x(scale))
    }

    pub fn scale_y(&self, scale: f64) -> Self{
        Self::new(self.position(), self.size().scale_y(scale))
    }

    pub fn offset(&self, delta: Vector) -> Self {
        Self::new(self.position()+delta, self.size())
    }
}

impl From<Rectangle<i32>> for Rectangle<f64> {
    fn from(value: Rectangle<i32>) -> Self {
        Self {
            height: value.height as f64,
            width: value.width as f64,
            x: value.x as f64,
            y: value.y as f64
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /*#[test]
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