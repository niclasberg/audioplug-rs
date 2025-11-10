use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::{
    BrushRef, Color, ColorMap, Ellipse, FillRule, Path, Rect, RoundedRect, Vec2f, Vec4f,
};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GpuShapeRef {
    /// bits 0-2: Shape type
    /// bit 3: Fill rule (path only): 0 -> even-odd, 1 -> non-zero
    /// bits 4-31: Number of segments (path only)
    shape_type: u32,
    /// ShapeData index or offset to first line segment
    index: u32,
}

pub enum GpuFill {
    Solid(Color),
    LinearGradient {
        start: Vec2f,
        end: Vec2f,
        color_stops: ColorMap,
    },
}

pub struct GpuScene {
    pub line_segments: Vec<LineSegment>,
    pub shapes: Vec<GpuShape>,
    pub fill_ops: Vec<u32>,
    pub gradient_lut: Vec<f32>,
}

impl GpuScene {
    const SHAPE_TYPE_PATH: u32 = 1;
    const SHAPE_TYPE_RECT: u32 = 2;
    const SHAPE_TYPE_ROUNDED_RECT: u32 = 3;
    const SHAPE_TYPE_ELLIPSE: u32 = 4;

    const FILL_RULE_EVEN_ODD: u32 = 1 << 3;

    const FILL_TYPE_SOLID: u32 = 1;
    const FILL_TYPE_LINEAR_GRADIENT: u32 = 2;
    const FILL_TYPE_RADIAL_GRADIENT: u32 = 3;

    pub fn new() -> Self {
        Self {
            line_segments: Vec::new(),
            shapes: Vec::new(),
            fill_ops: Vec::new(),
            gradient_lut: Vec::new(),
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

    pub fn fill_shape(&mut self, shape_ref: GpuShapeRef, fill: GpuFill) {
        let fill_type = match fill {
            GpuFill::Solid(_) => Self::FILL_TYPE_SOLID,
            GpuFill::LinearGradient { .. } => Self::FILL_TYPE_LINEAR_GRADIENT,
        };

        let fill_data = (shape_ref.shape_type) << 4 | fill_type;
        self.fill_ops.push(fill_data);
        self.fill_ops.push(shape_ref.index);

        match fill {
            GpuFill::Solid(color) => self.fill_ops.extend(
                [
                    (color.a * color.r).to_bits(),
                    (color.a * color.g).to_bits(),
                    (color.a * color.b).to_bits(),
                    color.a.to_bits(),
                ]
                .iter(),
            ),
            GpuFill::LinearGradient {
                start,
                end,
                color_stops,
            } => self.fill_ops.extend(
                [
                    start.x.to_bits(),
                    start.y.to_bits(),
                    end.x.to_bits(),
                    end.y.to_bits(),
                ]
                .iter(),
            ),
        };
    }

    pub fn shapes_data_byte_size(&self) -> u64 {
        self.shapes.len() * std::mem::size_of::<GpuShape>() as _
    }

    pub fn clear(&mut self) {
        self.fill_ops.clear();
        self.line_segments.clear();
        self.shapes.clear();
        self.gradient_lut.clear();
    }

    pub fn upload(
        &self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        shapes_data_buffer: &mut wgpu::Buffer,
        line_segments_buffer: &mut wgpu::Buffer,
        fill_ops_buffer: &mut wgpu::Buffer,
    ) {
        update_buffer(
            device,
            queue,
            shapes_data_buffer,
            bytemuck::cast_slice(&self.shapes),
        );
    }
}

fn update_buffer(
    device: &mut wgpu::Device,
    queue: &mut wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    data: &[u8],
) {
    if buffer.size() < data.len() as u64 {
        queue.write_buffer(&buffer, 0, data);
    } else {
        *buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Label"),
            contents: data,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
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
