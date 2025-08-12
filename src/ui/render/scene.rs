use crate::{
    core::{Point, Rect, Transform},
    ui::{BrushRef, ShapeRef, TextLayout},
};

pub struct Scene {}

impl Scene {
    pub fn new() -> Self {
        Self {}
    }

    pub fn fill<'c, 'd>(&mut self, shape: impl Into<ShapeRef<'c>>, brush: impl Into<BrushRef<'d>>) {
        //self.renderer.fill_shape(shape.into(), brush.into());
    }

    pub fn stroke<'c, 'd>(
        &mut self,
        shape: impl Into<ShapeRef<'c>>,
        brush: impl Into<BrushRef<'d>>,
        line_width: f32,
    ) {
        //self.renderer.stroke_shape(shape.into(), brush.into(), line_width);
    }

    pub fn draw_line<'c>(
        &mut self,
        p0: Point,
        p1: Point,
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        //self.renderer.draw_line(p0, p1, brush.into(), line_width)
    }

    pub fn draw_lines<'c>(
        &mut self,
        points: &[Point],
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        /*let brush = brush.into();
        for p in points.windows(2) {
            self.renderer.draw_line(p[0], p[1], brush, line_width)
        }*/
    }

    pub fn draw_bitmap(&mut self, source: &crate::platform::Bitmap, rect: impl Into<Rect>) {
        //self.renderer.draw_bitmap(source, rect.into())
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        //self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn use_clip(&mut self, rect: impl Into<Rect>, f: impl FnOnce(&mut Self)) {
        /*self.renderer.save();
        self.renderer.clip(rect.into());
        f(self);
        self.renderer.restore();*/
    }

    pub fn transform(&mut self, transform: impl Into<Transform>) {
        //self.renderer.transform(transform.into());
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {}
    }
}
