use bytemuck::{Pod, Zeroable};

use crate::core::{Ellipse, Rect, Vec4f};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LineSegment {
    p0: f32,
    p1: f32,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuShape {
    bounds: Rect<f32>,
    /// Corner radius, only for rounded rects
    radii: Vec4f,
    shape_type: u32,
}

impl GpuShape {
    const TYPE_RECT: u32 = 0;
    const TYPE_ROUNDED_RECT: u32 = 1;
    const TYPE_ELLIPSE: u32 = 2;
}

impl From<Rect<f32>> for GpuShape {
    fn from(value: Rect<f32>) -> Self {
        Self {
            bounds: value,
            radii: Vec4f::ZERO,
            shape_type: Self::TYPE_RECT,
        }
    }
}

impl From<Ellipse> for GpuShape {
    fn from(value: Ellipse) -> Self {
        Self {
            bounds: value.bounds().into(),
            radii: Vec4f::ZERO,
            shape_type: Self::TYPE_ELLIPSE,
        }
    }
}

pub struct Paint {}

impl Paint {
    const TYPE_SOLID: u32 = 1;
    const TYPE_LINEAR_GRADIENT: u32 = 2;
    const TYPE_RADIAL_GRADIENT: u32 = 3;
}

pub struct GpuScene {
    line_segments: Vec<LineSegment>,
}
