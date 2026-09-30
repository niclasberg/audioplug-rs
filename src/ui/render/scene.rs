use bytemuck::{NoUninit, Pod, Zeroable};

use crate::{
    core::{
        Color, CubicBezier, Ellipse, FillRule, FxHashMap, Line, LinearGradient, Paint, PaintRef,
        PathSegment, Point, QuadBezier, Rect, RoundedRect, ScaleFactor, ShadowKind, ShadowOptions,
        Shape, TextLayout, TranslateScale, Vec2, Zero,
    },
    ui::render::GlyphCache,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GpuShapeRef {
    /// bits 0-2: Shape type
    /// bit 3: Fill rule (path only): 0 -> even-odd, 1 -> non-zero
    /// bits 4-31: Number of segments (path only)
    shape_type: u32,
    /// ShapeData index or offset to first line segment
    index: u32,
    bounds: Rect,
}

impl GpuShapeRef {
    const SHAPE_TYPE_PATH: u32 = 1;
    const SHAPE_TYPE_RECT: u32 = 2;
    const SHAPE_TYPE_ROUNDED_RECT: u32 = 3;
    const SHAPE_TYPE_ELLIPSE: u32 = 4;

    const FILL_RULE_EVEN_ODD: u32 = 1 << 3;

    fn path(fill_rule: FillRule, segment_index: usize, len: usize, bounds: Rect) -> GpuShapeRef {
        let mut shape_type = Self::SHAPE_TYPE_PATH;
        if fill_rule == FillRule::EvenOdd {
            shape_type |= Self::FILL_RULE_EVEN_ODD;
        }

        assert!(len < u16::MAX as usize, "Path size too large");
        shape_type |= (len as u32) << 4;

        let index = segment_index as _;
        GpuShapeRef {
            shape_type,
            index,
            bounds,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GpuAppearanceRef(u32);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuDrawCommand {
    pub shape_type: u32,
    pub shape_index: u32,
    pub appearance_index: u32,
    pub scale: f32,
    pub translation: Vec2,
    pub bounds: Rect,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GpuAppearance {
    pub fill: GpuPaint,
    pub stroke: GpuPaint,
    pub shadow: GpuShadow,
    // ...bevel
    pub stroke_width: f32,
    pub flags: u32,
    pub margin: f32,
    _padding: [u32; 1],
}

impl GpuAppearance {
    pub const FILL_FLAG: u32 = 1;
    pub const STROKE_FLAG: u32 = 1 << 1;
    pub const SHADOW_FLAG: u32 = 1 << 2;

    pub fn new() -> Self {
        Self {
            fill: GpuPaint::empty(),
            stroke: GpuPaint::empty(),
            shadow: GpuShadow::empty(),
            stroke_width: 0.0,
            flags: 0,
            margin: 0.0,
            _padding: [0; 1],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.flags == 0
    }

    pub fn set_fill(&mut self, paint: GpuPaint) {
        self.fill = paint;
        self.flags |= Self::FILL_FLAG;
        self.margin = self.margin.max(0.5);
    }

    pub fn set_shadow(&mut self, shadow: ShadowOptions, scale_factor: ScaleFactor) {
        // Pre-multiply alpha
        let mut color = shadow.color;
        color.r *= color.a;
        color.g *= color.a;
        color.b *= color.a;
        let radius = shadow.radius * scale_factor.0;
        let offset = shadow.offset.scale(scale_factor.0);
        self.shadow = GpuShadow {
            kind: match shadow.kind {
                ShadowKind::DropShadow => GpuShadow::KIND_OUTER,
                ShadowKind::InnerShadow => GpuShadow::KIND_INNER,
            },
            radius,
            offset,
            color,
        };
        self.margin = self.margin.max(3.0 * radius + offset.abs().max_element());
        self.flags |= Self::SHADOW_FLAG;
    }

    pub fn set_stroke(&mut self, paint: GpuPaint, width: f32) {
        self.stroke = paint;
        self.stroke_width = width;
        self.margin = self.margin.max(width);
        self.flags |= Self::STROKE_FLAG;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Pod, Zeroable)]
pub struct GpuPaint {
    kind: u32,
    data: [u32; 7],
}

impl GpuPaint {
    pub const KIND_NONE: u32 = 1;
    pub const KIND_SOLID: u32 = 1;
    pub const KIND_LINEAR_GRADIENT: u32 = 2;
    pub const KIND_RADIAL_GRADIENT: u32 = 3;
    pub const KIND_IMAGE: u32 = 4;

    pub fn empty() -> Self {
        Self {
            kind: Self::KIND_NONE,
            data: [0; _],
        }
    }

    pub fn solid(color: Color) -> Self {
        let mut data = [0; _];
        data[..4].copy_from_slice(&color_to_bytes(color));
        Self {
            kind: Self::KIND_SOLID,
            data,
        }
    }

    pub fn linear_gradient(grad: &LinearGradient) -> Self {
        let data = [
            grad.start.x.to_bits(),
            grad.start.y.to_bits(),
            grad.end.x.to_bits(),
            grad.end.y.to_bits(),
            0,
            0,
            0,
        ];
        Self {
            kind: Self::KIND_LINEAR_GRADIENT,
            data,
        }
    }
}

impl From<Paint> for GpuPaint {
    fn from(value: Paint) -> Self {
        match value {
            Paint::Solid(color) => Self::solid(color),
            Paint::LinearGradient(linear_gradient) => Self::linear_gradient(&linear_gradient),
        }
    }
}

impl From<PaintRef<'_>> for GpuPaint {
    fn from(value: PaintRef<'_>) -> Self {
        match value {
            PaintRef::Solid(color) => Self::solid(color),
            PaintRef::LinearGradient(linear_gradient) => Self::linear_gradient(linear_gradient),
        }
    }
}

fn color_to_bytes(color: Color) -> [u32; 4] {
    // Pre-multiply alpha
    [
        (color.a * color.r).to_bits(),
        (color.a * color.g).to_bits(),
        (color.a * color.b).to_bits(),
        color.a.to_bits(),
    ]
}

impl Default for GpuPaint {
    fn default() -> Self {
        Self::empty()
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GpuShadow {
    pub kind: u32,
    pub radius: f32,
    pub offset: Vec2,
    pub color: Color,
}

impl GpuShadow {
    pub const KIND_NONE: u32 = 0;
    pub const KIND_OUTER: u32 = 1;
    pub const KIND_INNER: u32 = 2;

    pub fn empty() -> Self {
        Self {
            kind: Self::KIND_NONE,
            radius: 0.0,
            offset: Vec2::ZERO,
            color: Color::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit, Zeroable)]
pub struct GpuPathSegment {
    start: Point,
    end: Point,
    control1: Point,
    control2: Point,
    kind: GpuPathSegmentKind,
    /// Padding needed to fullfill wgsl alignment requirements
    _padding: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, NoUninit, Zeroable)]
pub enum GpuPathSegmentKind {
    Line = 0,
    Quad,
    Cubic,
}

impl GpuPathSegment {
    pub const EMPTY: Self = Self::line(Point::ZERO, Point::ZERO);

    pub const fn line(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            control1: Point::ZERO,
            control2: Point::ZERO,
            kind: GpuPathSegmentKind::Line,
            _padding: 0,
        }
    }

    pub fn quad(start: Point, control: Point, end: Point) -> Self {
        Self {
            start,
            end,
            control1: control,
            control2: Point::ZERO,
            kind: GpuPathSegmentKind::Quad,
            _padding: 0,
        }
    }

    pub fn cubic(start: Point, control1: Point, control2: Point, end: Point) -> Self {
        Self {
            start,
            end,
            control1,
            control2,
            kind: GpuPathSegmentKind::Cubic,
            _padding: 0,
        }
    }

    pub fn bounds(&self) -> Rect {
        match self.kind {
            GpuPathSegmentKind::Line => Line::new(self.start, self.end).bounds(),
            GpuPathSegmentKind::Quad => {
                QuadBezier::new(self.start, self.control1, self.end).bounds()
            }
            GpuPathSegmentKind::Cubic => {
                CubicBezier::new(self.start, self.control1, self.control2, self.end).bounds()
            }
        }
    }
}

#[derive(Default)]
pub struct GpuScene {
    pub shape_data: Vec<f32>,
    pub appearances: Vec<GpuAppearance>,
    pub draw_commands: Vec<GpuDrawCommand>,
    pub segments: Vec<GpuPathSegment>,
    pub segment_bounds: Vec<Rect>,
    pub gradient_lut: Vec<f32>,
    glyph_shapes: FxHashMap<usize, GpuShapeRef>,
}

impl GpuScene {
    pub const NOOP_FILL: [u32; 2] = [0, 0];

    pub fn new() -> Self {
        Self {
            shape_data: Vec::new(),
            draw_commands: Vec::new(),
            gradient_lut: Vec::new(),
            appearances: Vec::new(),
            segments: Vec::new(),
            segment_bounds: Vec::new(),
            glyph_shapes: Default::default(),
        }
    }

    pub fn add_rect(&mut self, rect: Rect) -> GpuShapeRef {
        self.add_shape_with_data(
            GpuShapeRef::SHAPE_TYPE_RECT,
            [
                rect.left as _,
                rect.top as _,
                rect.right as _,
                rect.bottom as _,
            ],
            rect,
        )
    }

    fn add_shape_with_data<const N: usize>(
        &mut self,
        shape_type: u32,
        values: [f32; N],
        bounds: Rect,
    ) -> GpuShapeRef {
        let index = self.shape_data.len() as u32;
        self.shape_data.extend(values.iter());
        GpuShapeRef {
            shape_type,
            index,
            bounds,
        }
    }

    pub fn add_rounded_rect(&mut self, rounded_rect: RoundedRect) -> GpuShapeRef {
        self.add_shape_with_data(
            GpuShapeRef::SHAPE_TYPE_ROUNDED_RECT,
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
            rounded_rect.bounds(),
        )
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) -> GpuShapeRef {
        self.add_shape_with_data(
            GpuShapeRef::SHAPE_TYPE_ELLIPSE,
            [
                ellipse.center.x as _,
                ellipse.center.y as _,
                ellipse.radii.width as _,
                ellipse.radii.height as _,
            ],
            ellipse.bounds(),
        )
    }

    pub fn add_shape(&mut self, shape: Shape, scale_factor: ScaleFactor) -> GpuShapeRef {
        match shape.scale(scale_factor.logical_to_physical()) {
            Shape::Rect(rect) => self.add_rect(rect),
            Shape::Rounded(rounded_rect) => self.add_rounded_rect(rounded_rect),
            Shape::Ellipse(ellipse) => self.add_ellipse(ellipse),
        }
    }

    pub fn add_path(
        &mut self,
        segments: &[PathSegment],
        fill_rule: FillRule,
        transform: TranslateScale,
    ) -> GpuShapeRef {
        let segment_index = self.segments.len();
        let mut shape_bounds = Rect::EMPTY;
        for segment in segments.iter() {
            let (gpu_segment, bounds) = match segment {
                PathSegment::Line(l) => (
                    GpuPathSegment::line(transform * l.p0, transform * l.p1),
                    transform * l.bounds(),
                ),
                PathSegment::Quad(q) => (
                    GpuPathSegment::quad(transform * q.p0, transform * q.p1, transform * q.p2),
                    transform * q.bounds(),
                ),
                PathSegment::Cubic(c) => (
                    GpuPathSegment::cubic(
                        transform * c.p0,
                        transform * c.p1,
                        transform * c.p2,
                        transform * c.p3,
                    ),
                    transform * c.bounds(),
                ),
            };
            self.segments.push(gpu_segment);
            self.segment_bounds.push(bounds);
            shape_bounds = shape_bounds.union(&bounds);
        }
        let segment_count = self.segments.len() - segment_index;
        GpuShapeRef::path(fill_rule, segment_index, segment_count, shape_bounds)
    }

    pub fn draw_text(
        &mut self,
        pos: Point,
        text_layout: &TextLayout,
        glyph_cache: &mut GlyphCache,
        scale_factor: ScaleFactor,
    ) {
        let mut app = GpuAppearance::new();
        app.set_stroke(GpuPaint::solid(Color::BLACK), 2.0);
        let app_ref = self.add_appearance(app);

        for line in text_layout.lines() {
            for item in line.items() {
                match item {
                    parley::PositionedLayoutItem::GlyphRun(glyph_run) => {
                        let mut run_x = pos.x + glyph_run.offset();
                        let run_y = pos.y + glyph_run.baseline();

                        let normalized_coords = glyph_run.run().normalized_coords();
                        let font = glyph_run.run().font();
                        let font_size = glyph_run.run().font_size();

                        for glyph in glyph_run.glyphs() {
                            let glyph_x = run_x + glyph.x;
                            let glyph_y = run_y + glyph.y;
                            run_x += glyph.advance;
                            let transform = TranslateScale::scale(scale_factor.0)
                                * TranslateScale {
                                    translation: Vec2::new(glyph_x, glyph_y),
                                    scale: font_size,
                                };

                            if let Some(cached_glyph) =
                                glyph_cache.get(font, glyph.id, normalized_coords)
                            {
                                let shape_ref = if let Some(shape_ref) =
                                    self.glyph_shapes.get(&cached_glyph.id)
                                {
                                    *shape_ref
                                } else {
                                    let segment_index = self.segments.len();
                                    self.segments.extend(cached_glyph.outline);
                                    self.segment_bounds.extend(cached_glyph.segment_bounds);

                                    let shape_ref = GpuShapeRef::path(
                                        FillRule::EvenOdd,
                                        segment_index,
                                        self.segments.len() - segment_index,
                                        cached_glyph.glyph_bounds,
                                    );
                                    self.glyph_shapes.insert(cached_glyph.id, shape_ref);
                                    shape_ref
                                };

                                self.draw(shape_ref, app_ref, transform);
                            }
                        }
                    }
                    parley::PositionedLayoutItem::InlineBox(_) => {}
                }
            }
        }
    }

    pub fn add_appearance(&mut self, appearance: GpuAppearance) -> GpuAppearanceRef {
        let apperarance_ref = GpuAppearanceRef(self.appearances.len() as _);
        self.appearances.push(appearance);
        apperarance_ref
    }

    pub fn draw(
        &mut self,
        shape_ref: GpuShapeRef,
        appearance_ref: GpuAppearanceRef,
        transform: TranslateScale,
    ) {
        assert!(appearance_ref.0 < self.appearances.len() as _);

        self.draw_commands.push(GpuDrawCommand {
            shape_type: shape_ref.shape_type,
            shape_index: shape_ref.index,
            appearance_index: appearance_ref.0,
            scale: transform.scale,
            translation: transform.translation,
            bounds: (transform * shape_ref.bounds)
                .inflate(self.appearances[appearance_ref.0 as usize].margin),
        });

        /*let fill_type = match fill {
            GpuFill::Solid(_) => Self::FILL_TYPE_SOLID,
            GpuFill::Stroke { .. } => Self::FILL_TYPE_STROKE,
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
            GpuFill::Solid(color) => self.fill_ops.extend(color_to_bytes(color).iter()),
            GpuFill::Stroke { color, width } => {
                self.fill_ops.extend(color_to_bytes(color).iter());
                self.fill_ops.push(width.to_bits());
            }
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
                color_stops: _,
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
                color_stops: _,
            } => self
                .fill_ops
                .extend([center.x.to_bits(), center.y.to_bits(), radius.to_bits()].iter()),
        };*/
    }

    pub fn clear(&mut self) {
        self.appearances.clear();
        self.draw_commands.clear();
        self.shape_data.clear();
        self.gradient_lut.clear();
        self.segments.clear();
        self.segment_bounds.clear();
        self.glyph_shapes.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.draw_commands.is_empty()
    }
}
