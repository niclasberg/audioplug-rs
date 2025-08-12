use windows::Win32::Graphics::Direct2D;

use crate::core::{Color, Ellipse, Point, Rect, RoundedRect, Size, Transform, Vec2};

impl From<Color> for Direct2D::Common::D2D1_COLOR_F {
    fn from(val: Color) -> Self {
        Direct2D::Common::D2D1_COLOR_F {
            r: val.r,
            g: val.g,
            b: val.b,
            a: val.a,
        }
    }
}

impl From<Rect> for Direct2D::Common::D2D_RECT_F {
    fn from(val: Rect) -> Self {
        Direct2D::Common::D2D_RECT_F {
            left: val.left() as f32,
            top: val.top() as f32,
            right: val.right() as f32,
            bottom: val.bottom() as f32,
        }
    }
}

impl From<Direct2D::Common::D2D_RECT_F> for Rect {
    fn from(value: Direct2D::Common::D2D_RECT_F) -> Self {
        Self::from_ltrb(
            value.left.into(),
            value.top.into(),
            value.right.into(),
            value.bottom.into(),
        )
    }
}

impl From<RoundedRect> for Direct2D::D2D1_ROUNDED_RECT {
    fn from(val: RoundedRect) -> Self {
        Direct2D::D2D1_ROUNDED_RECT {
            rect: val.rect.into(),
            radiusX: val.corner_radius.width as f32,
            radiusY: val.corner_radius.height as f32,
        }
    }
}

impl From<Ellipse> for Direct2D::D2D1_ELLIPSE {
    fn from(val: Ellipse) -> Self {
        Direct2D::D2D1_ELLIPSE {
            point: val.center.into(),
            radiusX: val.radii.width as f32,
            radiusY: val.radii.height as f32,
        }
    }
}

impl From<Point> for windows_numerics::Vector2 {
    fn from(val: Point) -> Self {
        windows_numerics::Vector2 {
            X: val.x as f32,
            Y: val.y as f32,
        }
    }
}

impl From<Vec2> for windows_numerics::Vector2 {
    fn from(val: Vec2) -> Self {
        windows_numerics::Vector2 {
            X: val.x as f32,
            Y: val.y as f32,
        }
    }
}

impl From<Transform> for windows_numerics::Matrix3x2 {
    fn from(val: Transform) -> Self {
        windows_numerics::Matrix3x2 {
            M11: val.m11 as f32,
            M12: val.m12 as f32,
            M21: val.m21 as f32,
            M22: val.m22 as f32,
            M31: val.tx as f32,
            M32: val.ty as f32,
        }
    }
}

impl From<windows_numerics::Matrix3x2> for Transform {
    fn from(value: windows_numerics::Matrix3x2) -> Self {
        Transform {
            m11: value.M11.into(),
            m12: value.M12.into(),
            m21: value.M21.into(),
            m22: value.M22.into(),
            tx: value.M31.into(),
            ty: value.M32.into(),
        }
    }
}

impl From<Size<u32>> for Direct2D::Common::D2D_SIZE_U {
    fn from(value: Size<u32>) -> Self {
        Direct2D::Common::D2D_SIZE_U {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Direct2D::Common::D2D_SIZE_F> for Size {
    fn from(value: Direct2D::Common::D2D_SIZE_F) -> Self {
        Self {
            width: value.width as _,
            height: value.height as _,
        }
    }
}
