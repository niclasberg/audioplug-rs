use icrate::Foundation::{CGRect, CGPoint, CGSize, CGFloat};

use crate::core::{Rectangle, Color, Point, Size, Vector};

use super::{core_graphics::{CGContext, CGColor, CGRef}, TextLayout};

impl Into<CGPoint> for Point {
    fn into(self) -> CGPoint {
        CGPoint { x: self.x, y: self.y }
    }
}

impl From<CGPoint> for Point {
    fn from(value: CGPoint) -> Self {
        Point::new(value.x, value.y)
    }
}

impl Into<CGSize> for Size {
    fn into(self) -> CGSize {
        CGSize { width: self.width, height: self.height }
    }
}

impl From<CGSize> for Size {
    fn from(value: CGSize) -> Self {
        Size::new(value.width, value.height)
    }
}

impl Into<CGRect> for Rectangle {
    fn into(self) -> CGRect {
        CGRect { origin: self.position().into(), size: self.size().into() }
    }
}

impl From<CGRect> for Rectangle {
    fn from(value: CGRect) -> Self {
        Rectangle::new(value.origin.into(), value.size.into())
    }
}

pub struct RendererRef<'a> {
	pub(super) context: &'a CGContext
}

impl<'a> RendererRef<'a> {
	pub fn set_offset(&mut self, delta: Vector) {
        self.context.translate(delta.x, delta.y)
    }

	pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        let color = to_cgcolor(color);
		self.context.set_fill_color(&color);
		self.context.stroke_rect(rect.into(), line_width.into());
    }

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        let color = to_cgcolor(color);
		self.context.set_fill_color(&color);
		println!("Render: {:?}", rect);println!("{:?}", rect);
		self.context.fill_rect(rect.into());
    }

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius: Size, color: Color) {
        let r: CGRect = rect.into();
		let min = r.min();
		let mid = r.mid();
		let max = r.max();

		println!("{:?}, {:?}, {:?}", min, mid, max);

		let color = to_cgcolor(color);
		self.context.set_fill_color(&color);

		self.context.move_to_point(min.x, mid.y); 
		// Add an arc through 2 to 3 
		self.context.add_arc_to_point(min.x, min.y, mid.x, min.y, radius.height); 
		// Add an arc through 4 to 5 
		self.context.add_arc_to_point(max.x, min.y, max.x, mid.y, radius.height); 
		// Add an arc through 6 to 7 
		self.context.add_arc_to_point(max.x, max.y, mid.x, max.y, radius.height); 
		// Add an arc through 8 to 9 
		self.context.add_arc_to_point(min.x, max.y, min.x, mid.y, radius.height); 
		// Close the path 
		self.context.close_path(); 
		// Fill & stroke the path 
		self.context.fill_path(); 
 
    }

	pub fn fill_ellipse(&mut self, origin: Point, radii: Size, color: Color) {
        let color = to_cgcolor(color);
		self.context.set_fill_color(&color);
		let rect = Rectangle::new(origin - Vector::new(radii.width, radii.height), radii.scale(2.0));
		self.context.fill_ellipse_in_rect(rect.into());
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point, color: Color) {
        
    }
}

fn to_cgcolor(color: Color) -> CGRef<CGColor> {
	CGColor::from_rgba(color.r.into(), color.g.into(), color.b.into(), color.a.into())
}