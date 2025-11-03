use bytemuck::{Pod, Zeroable};

use crate::core::{Color, Ellipse, FillRule, Path, Rect, RoundedRect, Vec2f, Vec4f};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LineSegment {
    p0: Vec2f,
    p1: Vec2f,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuShape {
    bounds: Vec4f,
    /// Corner radius, only for rounded rects
    radii: Vec4f,
}

pub struct Paint {}

impl Paint {
    const TYPE_SOLID: u32 = 1;
    const TYPE_LINEAR_GRADIENT: u32 = 2;
    const TYPE_RADIAL_GRADIENT: u32 = 3;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable, Pod)]
pub struct FillOp {
    color: Color,
    /// bits 0-2: Shape type
    /// bit 3: Fill rule (path only): 0 -> even-odd, 1 -> non-zero
    /// bits 4-31: Number of segments (path only)
    shape_type: u32,
    /// ShapeData index or offset to first line segment
    index: u32,
    _padding: u64,
}

pub struct GpuShapeRef {
    /// bits 0-2: Shape type
    /// bit 3: Fill rule (path only): 0 -> even-odd, 1 -> non-zero
    /// bits 4-31: Number of segments (path only)
    shape_type: u32,
    /// ShapeData index or offset to first line segment
    index: u32,
}

pub struct GpuScene {
    pub line_segments: Vec<LineSegment>,
    pub shapes: Vec<GpuShape>,
    pub fill_ops: Vec<FillOp>,
}

impl GpuScene {
    const SHAPE_TYPE_PATH: u32 = 1;
    const SHAPE_TYPE_RECT: u32 = 2;
    const SHAPE_TYPE_ROUNDED_RECT: u32 = 3;
    const SHAPE_TYPE_ELLIPSE: u32 = 4;

    const FILL_RULE_EVEN_ODD: u32 = 1 << 3;

    pub fn new() -> Self {
        Self {
            line_segments: Vec::new(),
            shapes: Vec::new(),
            fill_ops: Vec::new(),
        }
    }

    pub fn add_rect(&mut self, rect: Rect) -> GpuShapeRef {
        let index = self.shapes.len() as u32;
        self.shapes.push(GpuShape {
            bounds: rect_to_vec4f(rect),
            radii: Vec4f::ZERO,
        });
        GpuShapeRef {
            shape_type: Self::SHAPE_TYPE_RECT,
            index,
        }
    }

    pub fn add_rounded_rect(&mut self, rounded_rect: RoundedRect) -> GpuShapeRef {
        let index = self.shapes.len() as u32;
        let rect = rounded_rect.rect;
        self.shapes.push(GpuShape {
            bounds: rect_to_vec4f(rect),
            radii: Vec4f {
                x: rounded_rect.corner_radius.width as _,
                y: rounded_rect.corner_radius.height as _,
                z: rounded_rect.corner_radius.width as _,
                w: rounded_rect.corner_radius.height as _,
            },
        });
        GpuShapeRef {
            shape_type: Self::SHAPE_TYPE_ROUNDED_RECT,
            index,
        }
    }

    pub fn add_path(&mut self, path: &Path, fill_rule: FillRule) -> GpuShapeRef {
        let index = self.line_segments.len();
        path.flatten(1.0e-3, |line| {
            self.line_segments.push(LineSegment {
                p0: Vec2f {
                    x: line.p0.x as _,
                    y: line.p0.y as _,
                },
                p1: Vec2f {
                    x: line.p1.x as _,
                    y: line.p1.y as _,
                },
            });
        });

        let mut shape_type = Self::SHAPE_TYPE_PATH;
        if fill_rule == FillRule::EvenOdd {
            shape_type |= Self::FILL_RULE_EVEN_ODD;
        }

        let size = self.line_segments.len() - index;
        assert!(size < u16::MAX as usize, "Path size too large");
        shape_type |= (size as u32) << 4;

        let index = index as _;
        GpuShapeRef { shape_type, index }
    }

    pub fn fill_shape(&mut self, shape_ref: GpuShapeRef, color: Color) {
        self.fill_ops.push(FillOp {
            shape_type: shape_ref.shape_type,
            index: shape_ref.index,
            color,
            _padding: 0,
        });
    }

    pub fn clear(&mut self) {
        self.fill_ops.clear();
        self.line_segments.clear();
        self.shapes.clear();
    }
}

fn rect_to_vec4f(rect: Rect) -> Vec4f {
    Vec4f {
        x: rect.left as _,
        y: rect.top as _,
        z: rect.right as _,
        w: rect.bottom as _,
    }
}
