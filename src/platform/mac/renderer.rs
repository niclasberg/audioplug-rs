use objc2_core_text::CTFrameDraw;
use objc2_foundation::NSRect;
use objc2_core_foundation::{CGAffineTransform, CGFloat, CGRect};
use objc2_core_graphics::{CGColor, CGContext, CGContextAddArcToPoint, CGContextAddLineToPoint, CGContextClipToRect, CGContextClosePath, CGContextConcatCTM, CGContextFillEllipseInRect, CGContextFillPath, CGContextFillRect, CGContextGetCTM, CGContextMoveToPoint, CGContextRestoreGState, CGContextSaveGState, CGContextScaleCTM, CGContextSetFillColorWithColor, CGContextSetLineWidth, CGContextSetStrokeColorWithColor, CGContextStrokeEllipseInRect, CGContextStrokePath, CGContextStrokeRectWithWidth, CGContextTranslateCTM};
use crate::core::{Color, Point, Rectangle, RoundedRectangle, Size, Transform, Vector};

use super::{conversions::cgcolor_from_color, ImageSource, TextLayout};

pub struct RendererRef<'a> {
	pub(super) context: &'a CGContext,
	transforms: Vec<CGAffineTransform>,
	dirty_rect: Rectangle
}

impl<'a> RendererRef<'a> {
	pub fn new(context: &'a CGContext, dirty_rect: NSRect) -> Self {
		Self { 
			context, 
			transforms: Vec::new(),
			dirty_rect: dirty_rect.into()
		}
	}

	pub fn save(&mut self) {
		let ctm = unsafe { CGContextGetCTM(Some(&self.context)) };
		self.transforms.push(ctm);
		unsafe { CGContextSaveGState(Some(&self.context)) };
	}

	pub fn restore(&mut self) {
		if self.transforms.pop().is_some() {
			unsafe { CGContextRestoreGState(Some(&self.context)); }
		}
	}

	pub fn transform(&mut self, transform: Transform) {
		unsafe { CGContextConcatCTM(Some(&self.context), transform.into()) };
	}

	pub fn clip(&mut self, rect: Rectangle) {
		unsafe { CGContextClipToRect(Some(&self.context), rect.into()) }
	}

	pub fn set_offset(&mut self, delta: Vector) {
		unsafe { CGContextTranslateCTM(Some(&self.context), delta.x, delta.y) };
    }

	pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        self.set_stroke_color(color);
		unsafe { CGContextStrokeRectWithWidth(Some(&self.context), rect.into(), line_width.into()) };
    }

	pub fn draw_line(&mut self, p0: Point, p1: Point, color: Color, line_width: f32) {
		self.set_stroke_color(color);
		self.set_line_width(line_width);

		self.move_to_point(p0.x, p0.y); 
		self.add_line_to_point(p1.x, p1.y);
		unsafe { CGContextStrokePath(Some(&self.context)) };
	}

	pub fn draw_ellipse(&mut self, origin: Point, radii: Size, color: Color, line_width: f32) {
		self.set_stroke_color(color);
		self.set_line_width(line_width);

		let rect = Rectangle::new(origin - Vector::new(radii.width, radii.height), radii.scale(2.0));
		unsafe { CGContextStrokeEllipseInRect(Some(&self.context), rect.into()) }
	}

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        self.set_fill_color(color);
		unsafe { CGContextFillRect(Some(&self.context), rect.into()) }
    }

    pub fn fill_rounded_rectangle(&mut self, rect: RoundedRectangle, color: Color) {
		self.set_fill_color(color);
		self.add_rounded_rectangle(rect);
		unsafe { CGContextFillPath(Some(&self.context)) };
    }

	pub fn draw_rounded_rectangle(&mut self, rect: RoundedRectangle, color: Color, line_width: f32) {
		self.set_stroke_color(color);
		self.set_line_width(line_width);
		self.add_rounded_rectangle(rect);
		unsafe { CGContextStrokePath(Some(&self.context)) };
    }

	fn add_rounded_rectangle(&mut self, rect: RoundedRectangle) {
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

	fn move_to_point(&self, x: CGFloat, y: CGFloat) {
		unsafe { CGContextMoveToPoint(Some(&self.context), x, y) }
	}

	fn close_path(&self) {
		unsafe { CGContextClosePath(Some(&self.context)) }
	}

	fn add_arc_to_point(&self, x1: CGFloat, y1: CGFloat, x2: CGFloat, y2: CGFloat, radius: CGFloat) {
		unsafe { CGContextAddArcToPoint(Some(&self.context), x1, y1, x2, y2, radius) }
	}

	fn add_line_to_point(&self, x: CGFloat, y: CGFloat) {
		unsafe { CGContextAddLineToPoint(Some(&self.context), x, y) }
	}

	pub fn fill_ellipse(&mut self, origin: Point, radii: Size, color: Color) {
		self.set_fill_color(color);
		let rect = Rectangle::new(origin - Vector::new(radii.width, radii.height), radii.scale(2.0));
		unsafe { CGContextFillEllipseInRect(Some(&self.context), rect.into()) }
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
		let frame = text_layout.frame();
		let bounds = frame.bounding_box();

		unsafe { 
			CGContextSaveGState(Some(&self.context));
			CGContextTranslateCTM(Some(&self.context), position.x, position.y + bounds.size.height);
			CGContextScaleCTM(Some(&self.context), 1.0, -1.0);
			CTFrameDraw(&frame, &self.context);
			CGContextRestoreGState(Some(&self.context));
		};
    }

	pub fn draw_bitmap(&mut self, source: &ImageSource, rect: Rectangle) {
        unsafe { source.0.drawInRect(rect.into()) }
    }

	fn set_fill_color(&self, color: Color) {
		let color = cgcolor_from_color(color);
		unsafe { CGContextSetFillColorWithColor(Some(&self.context), Some(&color)) }
	}

	fn set_stroke_color(&self, color: Color) {
		let color = cgcolor_from_color(color);
		unsafe { CGContextSetStrokeColorWithColor(Some(&self.context), Some(&color)) };
	}

	fn set_line_width(&self, line_width: f32) {
		unsafe { CGContextSetLineWidth(Some(&self.context), line_width.into()) };
	}
}