use std::fmt::Debug;

use crate::core::{Circle, Ellipse, Point, Rect, RoundedRect, Size, Vec2, capsule::Capsule};

/// Represents a drawable shape
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Capsule(Capsule),
    Rect(Rect),
    Rounded(RoundedRect),
    Ellipse(Ellipse),
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

    pub const fn circle(center: Point, radius: f32) -> Self {
        Shape::Ellipse(Ellipse::new(center, Size::splat(radius)))
    }

    pub fn offset(&self, delta: impl Into<Vec2>) -> Self {
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.offset(delta)),
            Shape::Rounded(rect) => Shape::Rounded(rect.offset(delta)),
            Shape::Ellipse(ellipse) => Shape::Ellipse(ellipse.offset(delta)),
            Shape::Capsule(capsule) => Shape::Capsule(capsule.offset(delta)),
        }
    }

    pub fn bounds(&self) -> Rect {
        match self {
            Shape::Rect(rect) => *rect,
            Shape::Rounded(rounded) => rounded.bounds(),
            Shape::Ellipse(ell) => ell.bounds(),
            Shape::Capsule(capsule) => capsule.bounds(),
        }
    }

    pub fn inflate(self, amount: f32) -> Self {
        match self {
            Shape::Rect(rect) => Self::Rect(rect.inflate(amount)),
            Shape::Rounded(rounded_rect) => Self::Rounded(rounded_rect.inflate(amount)),
            Shape::Ellipse(ellipse) => Self::Ellipse(ellipse.inflate(amount)),
            Shape::Capsule(capsule) => Self::Capsule(capsule.inflate(amount)),
        }
    }

    pub fn scale(self, scale: f32) -> Self {
        match self {
            Shape::Rect(rect) => Self::Rect(rect.scale(scale)),
            Shape::Rounded(rounded_rect) => Self::Rounded(rounded_rect.scale(scale)),
            Shape::Ellipse(ellipse) => Self::Ellipse(ellipse.scale(scale)),
            Shape::Capsule(capsule) => Self::Capsule(capsule.scale(scale)),
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(pos),
            Shape::Rounded(rect) => rect.contains(pos),
            Shape::Ellipse(ell) => ell.contains(pos),
            Shape::Capsule(capsule) => capsule.contains(pos),
        }
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

impl From<Capsule> for Shape {
    fn from(value: Capsule) -> Self {
        Self::Capsule(value)
    }
}
