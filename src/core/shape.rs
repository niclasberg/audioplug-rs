use super::{Point, Rectangle, Size, Vector};

pub enum PathEl {
    MoveTo(Point),
    LineTo(Point)
}

/// Represents a drawable shape
/// All shapes are centered at (0, 0)
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Rect(Rectangle),
    RoundedRect { rect: Rectangle, corner_radius: Size},
    Ellipse { center: Point, radii: Size },
    Line { p0: Point, p1: Point}
}

impl Shape {
    pub const fn rect(point: Point, size: Size) -> Self {
        Shape::Rect(Rectangle::new(point, size))
    }

    pub const fn rounded_rect(point: Point, size: Size, corner_radius: Size) -> Self {
        Self::RoundedRect { rect: Rectangle::new(point, size), corner_radius }
    }

    pub const fn ellipse(center: Point, radii: Size) -> Self {
        Shape::Ellipse { center, radii }
    }

    pub const fn circle(center: Point, radius: f64) -> Self {
        Shape::Ellipse { center, radii: Size::new(radius, radius) }
    }

    pub const fn line(p0: Point, p1: Point) -> Self {
        Shape::Line { p0, p1 }
    }

    pub fn offset(self, delta: impl Into<Vector>) -> Self {
        let delta = delta.into();
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.offset(delta)),
            Shape::RoundedRect { rect, corner_radius } => Shape::RoundedRect { rect: rect.offset(delta), corner_radius },
            Shape::Ellipse { center, radii } => Shape::Ellipse { center: center + delta, radii },
            Shape::Line { p0, p1 } => Shape::Line { p0: p0 + delta, p1: p1 + delta }
        }
    }

    pub fn bounds(&self) -> Rectangle {
        match self {
            Shape::Rect(rect) => *rect,
            Shape::RoundedRect { rect,.. } => *rect,
            Shape::Ellipse { center, radii } => Rectangle::from_center(*center, radii.scale(2.0)),
            Shape::Line { p0, p1 } => Rectangle::from_points(*p0, *p1)
        }
    }

    pub fn hit_test(&self, pos: Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(pos),
            Shape::RoundedRect { rect, .. } => {
                if !rect.contains(pos) {
                    false
                } else {
                    // Check corners...
                    true
                }
            },
            Shape::Ellipse { center, radii } => {
                if radii.width < f64::EPSILON || radii.height < f64::EPSILON {
                    false
                } else {
                    ((pos.x - center.x) / radii.width).powi(2) + ((pos.y - center.y) / radii.height).powi(2) <= 1.0
                }
            },
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