use crate::core::{
    Color, ColorMap, Ellipse, FillRule, Path, Rect, RoundedRect, ShadowKind, ShadowOptions, Vec2f,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GpuShapeRef {
    /// bits 0-2: Shape type
    /// bit 3: Fill rule (path only): 0 -> even-odd, 1 -> non-zero
    /// bits 4-31: Number of segments (path only)
    shape_type: u32,
    /// ShapeData index or offset to first line segment
    index: u32,
}

#[derive(Debug, Clone)]
pub enum GpuFill {
    Solid(Color),
    Shadow(ShadowOptions),
    LinearGradient {
        start: Vec2f,
        end: Vec2f,
        color_stops: ColorMap,
    },
    RadialGradient {
        center: Vec2f,
        radius: f32,
        color_stops: ColorMap,
    },
}

pub struct GpuScene {
    pub shape_data: Vec<f32>,
    pub fill_ops: Vec<u32>,
    pub gradient_lut: Vec<f32>,
}

impl GpuScene {
    pub const NOOP_FILL: [u32; 2] = [0, 0];

    const SHAPE_TYPE_PATH: u32 = 1;
    const SHAPE_TYPE_RECT: u32 = 2;
    const SHAPE_TYPE_ROUNDED_RECT: u32 = 3;
    const SHAPE_TYPE_ELLIPSE: u32 = 4;

    const FILL_RULE_EVEN_ODD: u32 = 1 << 3;

    const FILL_TYPE_SOLID: u32 = 1;
    const FILL_TYPE_DROP_SHADOW: u32 = 2;
    const FILL_TYPE_INNER_SHADOW: u32 = 3;
    const FILL_TYPE_LINEAR_GRADIENT: u32 = 4;
    const FILL_TYPE_RADIAL_GRADIENT: u32 = 5;

    pub fn new() -> Self {
        Self {
            shape_data: Vec::new(),
            fill_ops: Vec::new(),
            gradient_lut: Vec::new(),
        }
    }

    pub fn add_rect(&mut self, rect: Rect) -> GpuShapeRef {
        self.add_shape_with_data(
            Self::SHAPE_TYPE_RECT,
            [
                rect.left as _,
                rect.top as _,
                rect.right as _,
                rect.bottom as _,
            ],
        )
    }

    fn add_shape_with_data<const N: usize>(
        &mut self,
        shape_type: u32,
        values: [f32; N],
    ) -> GpuShapeRef {
        let index = self.shape_data.len() as u32;
        self.shape_data.extend(values.iter());
        GpuShapeRef { shape_type, index }
    }

    pub fn add_rounded_rect(&mut self, rounded_rect: RoundedRect) -> GpuShapeRef {
        self.add_shape_with_data(
            Self::SHAPE_TYPE_ROUNDED_RECT,
            [
                rounded_rect.rect.left as _,
                rounded_rect.rect.top as _,
                rounded_rect.rect.right as _,
                rounded_rect.rect.bottom as _,
                rounded_rect.corner_radius.width as _,
                rounded_rect.corner_radius.height as _,
                rounded_rect.corner_radius.width as _,
                rounded_rect.corner_radius.height as _,
            ],
        )
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) -> GpuShapeRef {
        self.add_shape_with_data(
            Self::SHAPE_TYPE_ELLIPSE,
            [
                ellipse.center.x as _,
                ellipse.center.y as _,
                ellipse.radii.width as _,
                ellipse.radii.height as _,
            ],
        )
    }

    pub fn add_path(&mut self, path: &Path, fill_rule: FillRule) -> GpuShapeRef {
        let index = self.shape_data.len();
        path.flatten(1.0e-3, |line| {
            self.shape_data.extend(
                [
                    line.p0.x as _,
                    line.p0.y as _,
                    line.p1.x as _,
                    line.p1.y as _,
                ]
                .iter(),
            );
        });

        let mut shape_type = Self::SHAPE_TYPE_PATH;
        if fill_rule == FillRule::EvenOdd {
            shape_type |= Self::FILL_RULE_EVEN_ODD;
        }

        let size = (self.shape_data.len() - index) / 4;
        assert!(size < u16::MAX as usize, "Path size too large");
        shape_type |= (size as u32) << 4;

        let index = index as _;
        GpuShapeRef { shape_type, index }
    }

    pub fn fill_shape(&mut self, shape_ref: GpuShapeRef, fill: GpuFill) {
        let fill_type = match fill {
            GpuFill::Solid(_) => Self::FILL_TYPE_SOLID,
            GpuFill::Shadow(ShadowOptions { kind, .. }) => match kind {
                ShadowKind::DropShadow => Self::FILL_TYPE_DROP_SHADOW,
                ShadowKind::InnerShadow => Self::FILL_TYPE_INNER_SHADOW,
            },
            GpuFill::LinearGradient { .. } => Self::FILL_TYPE_LINEAR_GRADIENT,
            GpuFill::RadialGradient { .. } => Self::FILL_TYPE_RADIAL_GRADIENT,
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
            GpuFill::Shadow(ShadowOptions {
                radius,
                offset,
                color,
                ..
            }) => self.fill_ops.extend(
                [
                    (color.a * color.r).to_bits(),
                    (color.a * color.g).to_bits(),
                    (color.a * color.b).to_bits(),
                    color.a.to_bits(),
                    (offset.x as f32).to_bits(),
                    (offset.y as f32).to_bits(),
                    (radius as f32).to_bits(),
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
            GpuFill::RadialGradient {
                center,
                radius,
                color_stops,
            } => self
                .fill_ops
                .extend([center.x.to_bits(), center.y.to_bits(), radius.to_bits()].iter()),
        };
    }

    pub fn clear(&mut self) {
        self.fill_ops.clear();
        self.shape_data.clear();
        self.gradient_lut.clear();
    }
}
