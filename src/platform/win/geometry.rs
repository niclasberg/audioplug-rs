use windows::core::Result;
use windows::Win32::Graphics::Direct2D;
use crate::core::{Point, Rectangle, Transform};

use super::com::direct2d_factory;

#[derive(Clone)]
pub struct NativeGeometry(pub(super) Direct2D::ID2D1Geometry);

impl NativeGeometry {
    pub fn new(f: impl FnOnce(NativeGeometryBuilder) -> NativeGeometryBuilder) -> Result<Self> {
        let path = unsafe { direct2d_factory().CreatePathGeometry() }?;
        let sink = unsafe { path.Open() }?;

        let builder = f(NativeGeometryBuilder { sink, is_editing_path: false });
        if builder.is_editing_path {
            unsafe { builder.sink.EndFigure(Direct2D::Common::D2D1_FIGURE_END_OPEN) };
        }
        unsafe { builder.sink.Close() }?;

        Ok(Self(path.into()))
    }

    pub fn from_arc(center: Point, radius: f64, start_angle: f64, delta_angle: f64) -> Result<Self> {
        let start_point = center + Point::new(start_angle.cos(), start_angle.sin()).scale(radius);
        let end_angle = start_angle + delta_angle;
        let end_point = center + Point::new(end_angle.cos(), end_angle.sin()).scale(radius);
        Self::new(|builder| {
            let builder = builder.move_to(start_point.into());
            let arc = Direct2D::D2D1_ARC_SEGMENT {
                point: end_point.into(),
                size: todo!(),
                rotationAngle: 0.0,
                sweepDirection: todo!(),
                arcSize: todo!(),
            };
            unsafe { builder.sink.AddArc(&arc as *const _) };
            builder.close()
        })
    }

    pub fn transform(&self, transform: Transform) -> Result<Self> {
        let transform = transform.into();
        unsafe { direct2d_factory().CreateTransformedGeometry(&self.0, &transform as *const _) }
            .map(|transformed_geometry| Self(transformed_geometry.into()))   
    }

    pub fn bounds(&self) -> Result<Rectangle> {
        unsafe { self.0.GetBounds(None) }.map(|bounds| bounds.into())
    }
}

pub struct NativeGeometryBuilder {
    sink: Direct2D::ID2D1GeometrySink,
    is_editing_path: bool,
}

impl NativeGeometryBuilder {
    pub fn move_to(mut self, point: Point) -> Self {
        if self.is_editing_path {
            unsafe { self.sink.EndFigure(Direct2D::Common::D2D1_FIGURE_END_OPEN); };
        }

        unsafe { self.sink.BeginFigure(point.into(), Direct2D::Common::D2D1_FIGURE_BEGIN_FILLED) };
        self.is_editing_path = true;
        self
    }

    pub fn add_line_to(self, point: Point) -> Self {
        unsafe { self.sink.AddLine(point.into()) };
        self
    }

    pub fn add_cubic_curve_to(self, control_point1: Point, control_point2: Point, end: Point) -> Self {
        let bezier = Direct2D::Common::D2D1_BEZIER_SEGMENT {
            point1: control_point1.into(),
            point2: control_point2.into(),
            point3: end.into(),
        };
        unsafe { self.sink.AddBezier(&bezier as *const _) };
        self
    }

    pub fn add_quad_curve_to(self, control_point: Point, end: Point) -> Self {
        let bezier = Direct2D::D2D1_QUADRATIC_BEZIER_SEGMENT {
            point1: control_point.into(),
            point2: end.into(),
        };
        unsafe { self.sink.AddQuadraticBezier(&bezier as *const _) };
        self
    }

    pub fn add_arc_to(self, point: Point) -> Self {
        let arc = Direct2D::D2D1_ARC_SEGMENT {
            point: point.into(),
            size: todo!(),
            rotationAngle: todo!(),
            sweepDirection: todo!(),
            arcSize: todo!(),
        };

        unsafe { self.sink.AddArc(&arc as *const _) };
        self
    }

    pub fn close(mut self) -> Self {
        self.is_editing_path = false;
        unsafe { self.sink.EndFigure(Direct2D::Common::D2D1_FIGURE_END_CLOSED); };
        self
    }
}
