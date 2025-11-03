use std::fmt::Debug;

use crate::core::{Circle, Ellipse, Path, Point, Rect, RoundedRect, Size, Vec2};

/// Represents a drawable shape
#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rect),
    Rounded(RoundedRect),
    Ellipse(Ellipse),
    Path(Path),
}

impl Shape {
    pub fn rect(point: Point, size: Size) -> Self {
        Shape::Rect(Rect::from_origin(point, size))
    }

    pub fn rounded_rect(point: Point, size: Size, corner_radius: Size) -> Self {
        Self::Rounded(RoundedRect {
            rect: Rect::from_origin(point, size),
            corner_radius,
        })
    }

    pub const fn ellipse(center: Point, radii: Size) -> Self {
        Shape::Ellipse(Ellipse::new(center, radii))
    }

    pub const fn circle(center: Point, radius: f64) -> Self {
        Shape::Ellipse(Ellipse::new(center, Size::new(radius, radius)))
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        let delta = delta.into();
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.offset(delta)),
            Shape::Rounded(rect) => Shape::Rounded(rect.offset(delta)),
            Shape::Ellipse(ellipse) => Shape::Ellipse(ellipse.offset(delta)),
            Shape::Path(path) => Shape::Path(path.clone().offset(delta)),
        }
    }

    pub fn bounds(&self) -> Rect {
        match self {
            Shape::Rect(rect) => *rect,
            Shape::Rounded(rounded) => rounded.bounds(),
            Shape::Ellipse(ell) => ell.bounds(),
            Shape::Path(geometry) => geometry.bounds(),
        }
    }

    pub fn hit_test(&self, pos: Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(pos),
            Shape::Rounded(rect) => rect.contains(pos),
            Shape::Ellipse(ell) => ell.contains(pos),
            Shape::Path(_) => todo!(),
        }
    }
}

impl From<Path> for Shape {
    fn from(value: Path) -> Self {
        Self::Path(value)
    }
}

impl From<Rect> for Shape {
    fn from(value: Rect) -> Self {
        Self::Rect(value)
    }
}

impl From<RoundedRect> for Shape {
    fn from(value: RoundedRect) -> Self {
        Self::Rounded(value)
    }
}

impl From<Ellipse> for Shape {
    fn from(value: Ellipse) -> Self {
        Self::Ellipse(value)
    }
}

impl From<Circle> for Shape {
    fn from(value: Circle) -> Self {
        Self::Ellipse(value.into())
    }
}

#[derive(Clone, Copy)]
pub enum ShapeRef<'a> {
    Rect(Rect),
    Rounded(RoundedRect),
    Ellipse(Ellipse),
    Path(&'a Path),
}

impl ShapeRef<'_> {
    pub fn bounds(&self) -> Rect {
        match self {
            Self::Rect(rect) => *rect,
            Self::Rounded(rounded) => rounded.bounds(),
            Self::Ellipse(ell) => ell.bounds(),
            Self::Path(path) => path.bounds(),
        }
    }
}

impl<'a> From<&'a Shape> for ShapeRef<'a> {
    fn from(value: &'a Shape) -> Self {
        match value {
            Shape::Rect(rectangle) => Self::Rect(*rectangle),
            Shape::Rounded(rounded_rectangle) => Self::Rounded(*rounded_rectangle),
            Shape::Ellipse(ellipse) => Self::Ellipse(*ellipse),
            Shape::Path(path) => Self::Path(path),
        }
    }
}

impl<'a> From<Rect> for ShapeRef<'a> {
    fn from(value: Rect) -> Self {
        Self::Rect(value)
    }
}

impl<'a> From<RoundedRect> for ShapeRef<'a> {
    fn from(value: RoundedRect) -> Self {
        Self::Rounded(value)
    }
}

impl<'a> From<Ellipse> for ShapeRef<'a> {
    fn from(value: Ellipse) -> Self {
        Self::Ellipse(value)
    }
}

impl<'a> From<Circle> for ShapeRef<'a> {
    fn from(value: Circle) -> Self {
        Self::Ellipse(value.into())
    }
}

impl<'a> From<&'a Path> for ShapeRef<'a> {
    fn from(value: &'a Path) -> Self {
        Self::Path(value)
    }
}
