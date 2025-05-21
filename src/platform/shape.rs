use std::fmt::Debug;

use crate::core::{Circle, Ellipse, Point, Rectangle, RoundedRectangle, Size, Transform, Vector};
use crate::platform;

pub struct PathGeometryBuilder(platform::NativeGeometryBuilder);

impl PathGeometryBuilder {
    pub fn move_to(self, point: Point) -> Self {
        Self(self.0.move_to(point))
    }

    pub fn add_rectangle(self, rect: Rectangle) -> Self {
        self.move_to(rect.top_left())
            .add_line_to(rect.bottom_left())
            .add_line_to(rect.bottom_right())
            .add_line_to(rect.top_right())
            .close()
    }

    pub fn add_line_to(self, point: Point) -> Self {
        Self(self.0.add_line_to(point))
    }

    pub fn add_cubic_curve_to(
        self,
        control_point1: Point,
        control_point2: Point,
        end: Point,
    ) -> Self {
        Self(
            self.0
                .add_cubic_curve_to(control_point1, control_point2, end),
        )
    }

    pub fn add_quad_curve_to(self, control_point: Point, end: Point) -> Self {
        Self(self.0.add_quad_curve_to(control_point, end))
    }

    pub fn add_arc_to(self, point: Point) -> Self {
        Self(self.0.add_arc_to(point))
    }

    pub fn close(self) -> Self {
        Self(self.0.close())
    }
}

#[derive(Clone)]
pub struct PathGeometry(pub(crate) platform::NativeGeometry);

impl Debug for PathGeometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathGeometry").finish()
    }
}

impl PathGeometry {
    pub fn new(f: impl FnOnce(PathGeometryBuilder) -> PathGeometryBuilder) -> Self {
        let geometry =
            platform::NativeGeometry::new(move |builder| f(PathGeometryBuilder(builder)).0)
                .expect("Creating native geometry failed");
        Self(geometry)
    }

    pub fn with_transform(&self, transform: Transform) -> Self {
        Self(self.0.transform(transform).expect("Transform failed"))
    }

    pub fn bounds(&self) -> Rectangle {
        self.0.bounds().expect("Getting geometry bounds failed")
    }
}

/// Represents a drawable shape
#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rectangle),
    Rounded(RoundedRectangle),
    Ellipse(Ellipse),
    Geometry(PathGeometry),
}

impl Shape {
    pub const fn rect(point: Point, size: Size) -> Self {
        Shape::Rect(Rectangle::from_origin(point, size))
    }

    pub const fn rounded_rect(point: Point, size: Size, corner_radius: Size) -> Self {
        Self::Rounded(RoundedRectangle {
            rect: Rectangle::from_origin(point, size),
            corner_radius,
        })
    }

    pub const fn ellipse(center: Point, radii: Size) -> Self {
        Shape::Ellipse(Ellipse::new(center, radii))
    }

    pub const fn circle(center: Point, radius: f64) -> Self {
        Shape::Ellipse(Ellipse::new(center, Size::new(radius, radius)))
    }

    pub fn offset(&self, delta: impl Into<Vector>) -> Self {
        let delta = delta.into();
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.offset(delta)),
            Shape::Rounded(rect) => Shape::Rounded(rect.offset(delta)),
            Shape::Ellipse(ellipse) => Shape::Ellipse(ellipse.offset(delta)),
            Shape::Geometry(geometry) => {
                Shape::Geometry(geometry.with_transform(Transform::from_translation(delta)))
            }
        }
    }

    pub fn bounds(&self) -> Rectangle {
        match self {
            Shape::Rect(rect) => *rect,
            Shape::Rounded(rounded) => rounded.bounds(),
            Shape::Ellipse(ell) => ell.bounds(),
            Shape::Geometry(geometry) => geometry.bounds(),
        }
    }

    pub fn hit_test(&self, pos: Point) -> bool {
        match self {
            Shape::Rect(rect) => rect.contains(pos),
            Shape::Rounded(rect) => rect.contains(pos),
            Shape::Ellipse(ell) => ell.contains(pos),
            Shape::Geometry(_) => todo!(),
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

impl From<Circle> for Shape {
    fn from(value: Circle) -> Self {
        Self::Ellipse(value.into())
    }
}

pub enum ShapeRef<'a> {
    Rect(Rectangle),
    Rounded(RoundedRectangle),
    Ellipse(Ellipse),
    Geometry(&'a PathGeometry),
}

impl ShapeRef<'_> {
    pub fn bounds(&self) -> Rectangle {
        match self {
            Self::Rect(rect) => *rect,
            Self::Rounded(rounded) => rounded.bounds(),
            Self::Ellipse(ell) => ell.bounds(),
            Self::Geometry(geometry) => geometry.bounds(),
        }
    }
}

impl<'a> From<&'a Shape> for ShapeRef<'a> {
    fn from(value: &'a Shape) -> Self {
        match value {
            Shape::Rect(rectangle) => Self::Rect(*rectangle),
            Shape::Rounded(rounded_rectangle) => Self::Rounded(*rounded_rectangle),
            Shape::Ellipse(ellipse) => Self::Ellipse(*ellipse),
            Shape::Geometry(path_geometry) => Self::Geometry(path_geometry),
        }
    }
}

impl<'a> From<Rectangle> for ShapeRef<'a> {
    fn from(value: Rectangle) -> Self {
        Self::Rect(value)
    }
}

impl<'a> From<RoundedRectangle> for ShapeRef<'a> {
    fn from(value: RoundedRectangle) -> Self {
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

impl<'a> From<&'a PathGeometry> for ShapeRef<'a> {
    fn from(value: &'a PathGeometry) -> Self {
        Self::Geometry(value)
    }
}
