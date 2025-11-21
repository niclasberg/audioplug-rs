use crate::core::{Point, Transform, Vec2};

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
