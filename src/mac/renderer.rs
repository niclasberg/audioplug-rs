use icrate::Foundation::{CGRect, CGPoint, CGSize};

use crate::core::{Rectangle, Color, Point, Size, Vector};

use super::{core_graphics::{CGContext, CGColor, CGRef}, TextLayout};

impl Into<CGPoint> for Point {
    fn into(self) -> CGPoint {
        CGPoint { x: self.x, y: self.y }
    }
}

impl Into<CGSize> for Size {
    fn into(self) -> CGSize {
        CGSize { width: self.width, height: self.height }
    }
}

impl Into<CGRect> for Rectangle {
    fn into(self) -> CGRect {
        CGRect { origin: CGPoint::new(self.left(), self.bottom()), size: self.size().into() }
    }
}

pub(crate) struct RendererRef<'a> {
	context: &'a CGContext
}

impl<'a> RendererRef<'a> {
	pub fn set_offset(&mut self, delta: Vector) {
        todo!()
    }

	pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        let color = to_cgcolor(color);
		self.context.set_fill_color(&color);
		self.context.stroke_rect(rect.into(), line_width.into());
    }

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        let color = to_cgcolor(color);
		self.context.set_stroke_color(&color);
		self.context.fill_rect(rect.into());
    }

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius: Size, color: Color) {
        todo!()
    }

	pub fn fill_ellipse(&mut self, origin: Point, radii: Size, color: Color) {
        todo!()
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point, color: Color) {
        todo!()
    }
}

fn to_cgcolor(color: Color) -> CGRef<CGColor> {
	CGColor::from_rgba(color.r.into(), color.g.into(), color.b.into(), color.a.into())
}