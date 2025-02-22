use super::{Ellipse, Point, Rectangle, RoundedRectangle, Size, Vector};

pub enum PathEl {
    MoveTo(Point),
    LineTo(Point)
}


/// Represents a drawable shape
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Rect(Rectangle),
    Rounded(RoundedRectangle),
    Ellipse(Ellipse),
    Line { p0: Point, p1: Point}
}

impl Shape {
    pub const fn rect(point: Point, size: Size) -> Self {
        Shape::Rect(Rectangle::new(point, size))
    }

    pub const fn rounded_rect(point: Point, size: Size, corner_radius: Size) -> Self {
        Self::Rounded(RoundedRectangle { rect: Rectangle::new(point, size), corner_radius })
    }

    pub const fn ellipse(center: Point, radii: Size) -> Self {
        Shape::Ellipse(Ellipse::new(center, radii))
    }

    pub const fn circle(center: Point, radius: f64) -> Self {
        Shape::Ellipse(Ellipse::new(center, Size::new(radius, radius)))
    }

    pub const fn line(p0: Point, p1: Point) -> Self {
        Shape::Line { p0, p1 }
    }

    pub fn offset(self, delta: impl Into<Vector>) -> Self {
        let delta = delta.into();
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.offset(delta)),
            Shape::Rounded(rect) => Shape::Rounded(rect.offset(delta)),
            Shape::Ellipse(ellipse) => Shape::Ellipse(ellipse.offset(delta)),
            Shape::Line { p0, p1 } => Shape::Line { p0: p0 + delta, p1: p1 + delta }
        }
    }

    pub fn bounds(&self) -> Rectangle {
        match self {
            Shape::Rect(rect) => *rect,
            Shape::Rounded(RoundedRectangle { rect, ..}) => *rect,
            Shape::Ellipse(ell) => Rectangle::from_center(ell.center, ell.radii.scale(2.0)),
            Shape::Line { p0, p1 } => Rectangle::from_points(*p0, *p1)
        }
    }

    pub fn hit_test(&self, pos: Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(pos),
            Shape::Rounded(rect) => rect.contains(pos),
            Shape::Ellipse(ell) => ell.contains(pos),
            Shape::Line { .. } => {
                false
            }
        }
    }
}

impl From<Rectangle> for Shape {
    fn from(value: Rectangle) -> Self {
        Self::Rect(value)
    }
}

impl From<RoundedRectangle> for Shape {
    fn from(value: RoundedRectangle) -> Self {
        Self::Rounded(value)
    }
}

impl From<Ellipse> for Shape {
    fn from(value: Ellipse) -> Self {
        Self::Ellipse(value)
    }
}