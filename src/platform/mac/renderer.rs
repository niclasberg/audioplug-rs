use crate::{
    core::{
        Color, ColorMap, ColorStop, Point, Rectangle, RoundedRectangle, ShadowOptions, Transform,
        Vec2,
    },
    platform::{BrushRef, NativeTextLayout, ShapeRef},
    ui::LinearGradient,
};
use objc2_core_foundation::{CFRetained, CGAffineTransform, CGFloat, CGRect, CGSize};
use objc2_core_graphics::{
    CGColorSpace, CGContext, CGGradient, CGGradientDrawingOptions, CGMutablePath, CGPath,
};
use objc2_core_image::CIContext;
use objc2_core_text::CTFrame;
use objc2_foundation::NSRect;

use super::{Bitmap, Error, conversions::cgcolor_from_color};

pub struct RendererRef<'a> {
    pub(super) context: &'a CGContext,
    pub(super) ci_context: &'a CIContext,
    transforms: Vec<CGAffineTransform>,
    dirty_rect: NSRect,
}

impl<'a> RendererRef<'a> {
    pub fn new(context: &'a CGContext, ci_context: &'a CIContext, dirty_rect: NSRect) -> Self {
        Self {
            context,
            ci_context,
            transforms: Vec::new(),
            dirty_rect,
        }
    }

    pub fn dirty_rect(&self) -> Rectangle {
        self.dirty_rect.into()
    }

    pub fn save(&mut self) {
        let ctm = unsafe { CGContext::ctm(Some(&self.context)) };
        self.transforms.push(ctm);
        unsafe { CGContext::save_g_state(Some(&self.context)) };
    }

    pub fn restore(&mut self) {
        if self.transforms.pop().is_some() {
            unsafe {
                CGContext::restore_g_state(Some(&self.context));
            }
        }
    }

    pub fn transform(&mut self, transform: Transform) {
        unsafe { CGContext::concat_ctm(Some(&self.context), transform.into()) };
    }

    pub fn clip(&mut self, rect: Rectangle) {
        unsafe { CGContext::clip_to_rect(Some(&self.context), rect.into()) }
    }

    pub fn set_offset(&mut self, delta: Vec2) {
        unsafe { CGContext::translate_ctm(Some(&self.context), delta.x, delta.y) };
    }

    pub fn draw_shadow(&mut self, shape: ShapeRef, options: ShadowOptions) {
        // Y-axis is flipped, need to flip offset
        let offset = CGSize {
            width: options.offset.x,
            height: -options.offset.y,
        };

        // Outer bounds
        let bounds = shape
            .bounds()
            .expand(options.radius)
            .expand_x(offset.width.abs())
            .expand_y(offset.height.abs());

        self.save();

        // Seems like we need some color here, or the shadow won't render
        // Doesn't matter, we clip it anyway
        self.set_fill_color(Color::from_rgba8(0, 0, 0, 1.0));
        let shadow_color = cgcolor_from_color(options.color);
        unsafe {
            CGContext::set_shadow_with_color(
                Some(&self.context),
                offset,
                options.radius,
                Some(&shadow_color),
            )
        };

        match options.kind {
            crate::core::ShadowKind::DropShadow => {
                // Clip so that we only render the shadow outside the shape
                self.add_rectangle(bounds);
                self.add_shape(shape);
                unsafe { CGContext::eo_clip(Some(&self.context)) };

                self.do_fill_shape(shape);
            }
            crate::core::ShadowKind::InnerShadow => {
                // Only include inside of shape
                self.add_shape(shape);
                unsafe { CGContext::clip(Some(&self.context)) };

                self.add_rectangle(bounds);
                self.add_shape(shape);
                unsafe {
                    CGContext::eo_fill_path(Some(&self.context));
                }
            }
        }

        self.restore();
    }

    pub fn fill_shape(&mut self, shape: ShapeRef, brush: BrushRef) {
        match brush {
            BrushRef::Solid(color) => {
                self.set_fill_color(color);
                self.do_fill_shape(shape);
            }
            BrushRef::LinearGradient(linear_gradient) => {
                self.save();

                let bounds = shape.bounds();
                self.add_shape(shape);
                unsafe { CGContext::clip(Some(&self.context)) };
                self.draw_linear_gradient(linear_gradient, bounds);

                self.restore();
            }
        }
    }

    fn do_fill_shape(&self, shape: ShapeRef) {
        match shape {
            ShapeRef::Rect(rectangle) => unsafe {
                CGContext::fill_rect(Some(&self.context), rectangle.into())
            },
            ShapeRef::Rounded(rounded_rectangle) => {
                self.add_rounded_rectangle(rounded_rectangle);
                unsafe { CGContext::fill_path(Some(&self.context)) };
            }
            ShapeRef::Ellipse(ellipse) => {
                let rect = ellipse.bounds();
                unsafe { CGContext::fill_ellipse_in_rect(Some(&self.context), rect.into()) }
            }
            ShapeRef::Geometry(path_geometry) => unsafe {
                CGContext::add_path(Some(&self.context), Some(&path_geometry.0.0));
                CGContext::fill_path(Some(&self.context));
            },
        }
    }

    pub fn stroke_shape(&mut self, shape: ShapeRef, brush: BrushRef, line_width: f32) {
        match brush {
            BrushRef::Solid(color) => {
                self.set_line_width(line_width);
                self.set_stroke_color(color);
                match shape {
                    ShapeRef::Rect(rectangle) => unsafe {
                        CGContext::stroke_rect_with_width(
                            Some(&self.context),
                            rectangle.into(),
                            line_width.into(),
                        )
                    },
                    ShapeRef::Rounded(rounded_rectangle) => {
                        self.add_rounded_rectangle(rounded_rectangle);
                        unsafe { CGContext::stroke_path(Some(&self.context)) };
                    }
                    ShapeRef::Ellipse(ellipse) => unsafe {
                        CGContext::stroke_ellipse_in_rect(
                            Some(&self.context),
                            ellipse.bounds().into(),
                        )
                    },
                    ShapeRef::Geometry(path_geometry) => unsafe {
                        CGContext::add_path(Some(&self.context), Some(&path_geometry.0.0));
                        CGContext::stroke_path(Some(&self.context));
                    },
                }
            }
            BrushRef::LinearGradient(linear_gradient) => {
                self.save();
                self.set_line_width(line_width);
                let bounds = shape.bounds();
                self.add_shape(shape);

                unsafe {
                    CGContext::replace_path_with_stroked_path(Some(&self.context));
                    CGContext::clip(Some(&self.context));
                }
                self.draw_linear_gradient(linear_gradient, bounds);
                self.restore();
            }
        }
    }

    fn draw_linear_gradient(&self, gradient: &LinearGradient, bounds: Rectangle) {
        let start_point = gradient.start.resolve(bounds);
        let end_point = gradient.end.resolve(bounds);
        unsafe {
            CGContext::draw_linear_gradient(
                Some(&self.context),
                Some(&gradient.native.gradient),
                start_point.into(),
                end_point.into(),
                CGGradientDrawingOptions::empty(),
            );
        }
    }

    pub fn draw_line(&mut self, p0: Point, p1: Point, brush: BrushRef, line_width: f32) {
        self.set_line_width(line_width);
        match brush {
            BrushRef::Solid(color) => {
                self.set_stroke_color(color);
                self.add_line(p0, p1);
                unsafe { CGContext::stroke_path(Some(&self.context)) };
            }
            BrushRef::LinearGradient(linear_gradient) => {
                self.save();

                self.add_line(p0, p1);
                unsafe {
                    CGContext::replace_path_with_stroked_path(Some(&self.context));
                    CGContext::clip(Some(&self.context));
                }
                self.draw_linear_gradient(linear_gradient, Rectangle::from_points(p0, p1));

                self.restore();
            }
        }
    }

    pub fn draw_text(&mut self, text_layout: &NativeTextLayout, position: Point) {
        let frame = text_layout.frame();
        let bounds = frame.bounding_box();

        unsafe {
            self.save();
            CGContext::translate_ctm(
                Some(&self.context),
                position.x,
                position.y + bounds.size.height,
            );
            CGContext::scale_ctm(Some(&self.context), 1.0, -1.0);
            CTFrame::draw(&frame, &self.context);
            self.restore();
        };
    }

    pub fn draw_bitmap(&mut self, source: &Bitmap, rect: Rectangle) {
        unsafe { source.0.drawInRect(rect.into()) }
    }

    fn add_line(&mut self, p0: Point, p1: Point) {
        self.move_to_point(p0.x, p0.y);
        self.add_line_to_point(p1.x, p1.y);
    }

    fn add_rounded_rectangle(&self, rect: RoundedRectangle) {
        let r: CGRect = rect.rect.into();
        let min = r.min();
        let mid = r.mid();
        let max = r.max();

        self.move_to_point(min.x, mid.y);
        // Add an arc through 2 to 3
        self.add_arc_to_point(min.x, min.y, mid.x, min.y, rect.corner_radius.height);
        // Add an arc through 4 to 5
        self.add_arc_to_point(max.x, min.y, max.x, mid.y, rect.corner_radius.height);
        // Add an arc through 6 to 7
        self.add_arc_to_point(max.x, max.y, mid.x, max.y, rect.corner_radius.height);
        // Add an arc through 8 to 9
        self.add_arc_to_point(min.x, max.y, min.x, mid.y, rect.corner_radius.height);
        // Close the path
        self.close_path();
    }

    fn add_rectangle(&self, rect: Rectangle) {
        unsafe { CGContext::add_rect(Some(&self.context), rect.into()) }
    }

    fn add_shape(&self, shape: ShapeRef) {
        match shape {
            ShapeRef::Rect(rectangle) => self.add_rectangle(rectangle),
            ShapeRef::Rounded(rounded_rectangle) => self.add_rounded_rectangle(rounded_rectangle),
            ShapeRef::Ellipse(ellipse) => unsafe {
                CGContext::add_ellipse_in_rect(Some(&self.context), ellipse.bounds().into())
            },
            ShapeRef::Geometry(path_geometry) => unsafe {
                CGContext::add_path(Some(&self.context), Some(&path_geometry.0.0))
            },
        }
    }

    fn move_to_point(&self, x: CGFloat, y: CGFloat) {
        unsafe { CGContext::move_to_point(Some(&self.context), x, y) }
    }

    fn close_path(&self) {
        unsafe { CGContext::close_path(Some(&self.context)) }
    }

    fn add_arc_to_point(
        &self,
        x1: CGFloat,
        y1: CGFloat,
        x2: CGFloat,
        y2: CGFloat,
        radius: CGFloat,
    ) {
        unsafe { CGContext::add_arc_to_point(Some(&self.context), x1, y1, x2, y2, radius) }
    }

    fn add_line_to_point(&self, x: CGFloat, y: CGFloat) {
        unsafe { CGContext::add_line_to_point(Some(&self.context), x, y) }
    }

    fn set_fill_color(&self, color: Color) {
        let color = cgcolor_from_color(color);
        unsafe { CGContext::set_fill_color_with_color(Some(&self.context), Some(&color)) }
    }

    fn set_stroke_color(&self, color: Color) {
        let color = cgcolor_from_color(color);
        unsafe { CGContext::set_stroke_color_with_color(Some(&self.context), Some(&color)) };
    }

    fn set_line_width(&self, line_width: f32) {
        unsafe { CGContext::set_line_width(Some(&self.context), line_width.into()) };
    }
}

#[derive(Clone)]
pub struct NativeLinearGradient {
    pub color_map: ColorMap,
    gradient: CFRetained<CGGradient>,
}

impl NativeLinearGradient {
    pub fn new(color_map: ColorMap) -> Self {
        let gradient = create_gradient(&color_map);
        Self {
            color_map,
            gradient,
        }
    }
}

fn create_gradient(color_map: &ColorMap) -> CFRetained<CGGradient> {
    let mut components = Vec::new();
    let mut locations = Vec::new();
    for ColorStop { position, color } in color_map.stops.iter() {
        components.push(color.r as f64);
        components.push(color.g as f64);
        components.push(color.b as f64);
        components.push(color.a as f64);
        locations.push(*position as f64);
    }

    let space = unsafe { CGColorSpace::new_device_rgb() };
    unsafe {
        CGGradient::with_color_components(
            space.as_ref().map(|x| x.as_ref()),
            components.as_ptr(),
            locations.as_ptr(),
            locations.len(),
        )
    }
    .unwrap()
}

#[derive(Clone)]
pub struct NativeRadialGradient {
    pub color_map: ColorMap,
    gradient: CFRetained<CGGradient>,
}

impl NativeRadialGradient {
    pub fn new(color_map: ColorMap) -> Self {
        let gradient = create_gradient(&color_map);
        Self {
            color_map,
            gradient,
        }
    }
}

pub struct NativeGeometryBuilder(CFRetained<CGMutablePath>);

impl NativeGeometryBuilder {
    pub fn move_to(self, point: Point) -> Self {
        /*if self.is_editing_path {
            unsafe {
                self.sink.EndFigure(Direct2D::Common::D2D1_FIGURE_END_OPEN);
            };
        }

        unsafe {
            self.sink
                .BeginFigure(point.into(), Direct2D::Common::D2D1_FIGURE_BEGIN_FILLED)
        };
        self.is_editing_path = true;*/
        unsafe {
            CGMutablePath::move_to_point(Some(&self.0), std::ptr::null(), point.x, point.y);
        }

        self
    }

    pub fn add_line_to(self, point: Point) -> Self {
        unsafe {
            CGMutablePath::add_line_to_point(Some(&self.0), std::ptr::null(), point.x, point.y);
        }
        self
    }

    pub fn add_cubic_curve_to(
        self,
        control_point1: Point,
        control_point2: Point,
        end: Point,
    ) -> Self {
        unsafe {
            CGMutablePath::add_curve_to_point(
                Some(&self.0),
                std::ptr::null(),
                control_point1.x,
                control_point1.y,
                control_point2.x,
                control_point2.y,
                end.x,
                end.y,
            );
        }
        self
    }

    pub fn add_quad_curve_to(self, control_point: Point, end: Point) -> Self {
        unsafe {
            CGMutablePath::add_quad_curve_to_point(
                Some(&self.0),
                std::ptr::null(),
                control_point.x,
                control_point.y,
                end.x,
                end.y,
            );
        }
        self
    }

    pub fn add_arc_to(self, _point: Point) -> Self {
        /*let arc = Direct2D::D2D1_ARC_SEGMENT {
            point: point.into(),
            size: todo!(),
            rotationAngle: todo!(),
            sweepDirection: todo!(),
            arcSize: todo!(),
        };

        unsafe { self.sink.AddArc(&arc as *const _) };*/
        self
    }

    pub fn close(self) -> Self {
        /*self.is_editing_path = false;*/
        unsafe {
            CGMutablePath::close_subpath(Some(&self.0));
        }
        self
    }
}

#[derive(Clone)]
pub struct NativeGeometry(CFRetained<CGPath>);

impl NativeGeometry {
    pub fn new(
        f: impl FnOnce(NativeGeometryBuilder) -> NativeGeometryBuilder,
    ) -> Result<Self, Error> {
        let path = unsafe { CGMutablePath::new() };
        let builder = f(NativeGeometryBuilder(path));
        let dd = builder.0.downcast().map_err(|_| Error)?;
        Ok(Self(dd))
    }

    pub fn transform(&self, transform: Transform) -> Result<Self, Error> {
        let path =
            unsafe { CGPath::new_copy_by_transforming_path(Some(&self.0), &transform.into()) }
                .ok_or(Error)?;
        Ok(Self(path))
    }

    pub fn bounds(&self) -> Result<Rectangle, Error> {
        let bounds = unsafe { CGPath::bounding_box(Some(&self.0)) };
        Ok(bounds.into())
    }
}
