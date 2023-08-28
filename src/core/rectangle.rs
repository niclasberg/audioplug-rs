use super::Point;
use super::Size;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rectangle {
    pub fn new(point: Point, size: Size) -> Self {
        Self { x: point.x, y: point.y, width: size.width, height: size.height }
    }

    pub fn from_points(x0: Point, x1: Point) -> Self {
        Self { x: x0.x, y: x0.y, width: x1.x - x0.x, height: x1.y - x0.y }
    }

    pub fn from_xywh(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.left() && point.x <= self.right() &&
        point.y >= self.bottom() && point.y <= self.top()
    }

    pub fn intersects(&self, other: Rectangle) -> bool {
        !(
            self.left() > other.right() ||
            self.right() < other.left() ||
            self.top() < other.bottom() ||
            self.bottom() > other.bottom()
        )
    }

    pub fn left(&self) -> f64 {
        self.x
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn top(&self) -> f64 {
        self.y + self.height
    }

    pub fn bottom(&self) -> f64 {
        self.y
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_points() {
        let p0 = Point::new(1.0, 2.0);
        let p1 = Point::new(3.0, 4.0);
        let rect = Rectangle::from_points(p0, p1);

        assert_eq!(rect.left(), p0.x);
        assert_eq!(rect.right(), p1.x);
        assert_eq!(rect.bottom(), p0.y);
        assert_eq!(rect.top(), p1.y);
    }

    #[test]
    fn contains() {
        let rect = Rectangle::new(Point::new(1.0, 2.0), Size::new(15.0, 20.0));
        assert!(rect.contains(Point::new(1.0, 2.0)));
        assert!(rect.contains(Point::new(16.0, 22.0)));
        assert!(rect.contains(Point::new(10.0, 18.0)));
        
        assert!(!rect.contains(Point::new(1.0, 23.0)));
        assert!(!rect.contains(Point::new(1.0, 1.0)));
        
        assert!(!rect.contains(Point::new(0.0, 4.0)));
        assert!(!rect.contains(Point::new(17.0, 4.0)));
    }
}