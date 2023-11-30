use icrate::Foundation::{CGRect, CGPoint, CGSize, CGFloat};

use crate::core::{Rectangle, Color, Point, Size, Vector, Transform};

use super::{core_graphics::{CGContext, CGColor, CGPath, CGAffineTransform}, TextLayout, IRef};

pub struct RendererRef<'a> {
	pub(super) context: &'a CGContext,
	transforms: Vec<CGAffineTransform>
}

impl<'a> RendererRef<'a> {
	pub fn new(context: &'a CGContext) -> Self {
		Self { context, transforms: Vec::new() }
	}

	pub fn save(&mut self) {
		self.transforms.push(self.context.get_ctm());
		self.context.save_state();
	}

	pub fn restore(&mut self) {
		if self.transforms.pop().is_some() {
			self.context.restore_state();
		}
	}

	pub fn transform(&mut self, transform: Transform) {
		self.context.concat_ctm(transform.into());
	}

	pub fn set_offset(&mut self, delta: Vector) {
        self.context.translate_ctm(delta.x, delta.y)
    }

	pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color, line_width: f32) {
        let color = to_cgcolor(color);
		self.context.set_fill_color(&color);
		self.context.stroke_rect(rect.into(), line_width.into());
    }

	pub fn draw_line(&mut self, p0: Point, p1: Point, color: Color, line_width: f32) {
		todo!()
	}

    pub fn fill_rectangle(&mut self, rect: Rectangle, color: Color) {
        let color = to_cgcolor(color);
		self.context.set_fill_color(&color);
		self.context.fill_rect(rect.into());
    }

    pub fn fill_rounded_rectangle(&mut self, rect: Rectangle, radius: Size, color: Color) {
        let r: CGRect = rect.into();
		let min = r.min();
		let mid = r.mid();
		let max = r.max();

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
		let (string_range, size) = text_layout.suggested_range_and_size();
		let size = size.into();
		let rect = Rectangle::new(Point::ZERO, size).into();
		let path = CGPath::create_with_rect(rect, None);
		let frame = text_layout.frame_setter.create_frame(string_range, &path, None);

		self.context.save_state();

		self.context.translate_ctm(position.x, position.y + rect.size.height);
		self.context.scale_ctm(1.0, -1.0);
		frame.draw(self.context);

		self.context.restore_state();
    }
}

fn to_cgcolor(color: Color) -> IRef<CGColor> {
	CGColor::from_rgba(color.r.into(), color.g.into(), color.b.into(), color.a.into())
}