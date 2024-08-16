use objc2_foundation::{CGSize, CGPoint};

use crate::{text::FontWeight, core::{Color, Size, Point, Rectangle}};
use super::{core_text::{CTFrameSetter, CTFrame, AttributedStringBuilder, CTLine}, IRef, core_foundation::{CFRange, CFString, CFAttributedString, CFIndex}, IMut, core_graphics::{CGColor, CGPath}};

pub struct TextLayout{
    pub(super) attributed_string: IMut<CFAttributedString>,
	pub(super) frame_setter: IRef<CTFrameSetter>,
	text_frame: TextFrame,
	max_size: CGSize,
	text: String
}

struct TextLine {
	line: IRef<CTLine>,
	origin: CGPoint,
	char_range: CFRange
}

struct TextFrame {
	frame: IRef<CTFrame>,
	lines: Vec<TextLine>
}

impl TextFrame {
	fn new(frame_setter: &CTFrameSetter, string_range: CFRange, max_size: CGSize) -> Self {
		let (string_range, size) = frame_setter.suggest_frame_size_with_constraints(string_range, None, max_size.into());

		let rect = Rectangle::new(Point::ZERO, size.into()).into();
		let path = CGPath::create_with_rect(rect, None);
		let frame = frame_setter.create_frame(string_range, &path, None);

		let lines = frame.get_lines();
		let origins = frame.get_line_origins();
		let lines = lines.into_iter().zip(origins.into_iter())
			.map(|(line, origin)| {
				let char_range = line.string_range();
				TextLine { line, origin, char_range }
			}).collect();

		Self {
			frame,
			lines
		}
	}
}

impl TextLayout {
    pub fn new(
        string: &str, 
        _font_family: &str, 
        _font_weight: FontWeight,
        _font_size: f32,
        max_size: Size,
		color: Color
    ) -> Self {
		let text = string.to_string();
		let string = CFString::new(string);
		let mut builder = AttributedStringBuilder::new(&string);
		let color = CGColor::from_color(color);
		builder.set_foreground_color(builder.range(), &color);
		let attributed_string = builder.0;
		let frame_setter = CTFrameSetter::from_attributed_string(&attributed_string);

		let string_range = (0..attributed_string.length() as isize).into();
		let text_frame = TextFrame::new(&frame_setter, string_range, max_size.into());

        Self {
			attributed_string,
			frame_setter,
			text_frame,
			max_size: max_size.into(),
			text
		}
    }

    pub fn set_max_size(&mut self, size: Size) {
		let size = size.into();
		if self.max_size != size {
			self.max_size = size;
			let string_range = (0..self.attributed_string.length() as isize).into();
			self.text_frame = TextFrame::new(&self.frame_setter, string_range, size);
		}
    }

	pub fn measure(&self) -> Size {
		self.text_frame.frame.path().bounding_box().size.into()
	}

	pub fn text_index_at_point(&self, point: Point) -> Option<usize> {
        self.text_frame.lines.iter()
			.find_map(|TextLine { line, origin, char_range }| {
				let origin: Point = (*origin).into();
				let point: Point = point + origin;
				let point = (origin + point).into();
				let index = line.string_index_for_position(point);
				if index < 0 {
					None
				} else {
					let absolute_index = index + char_range.location;
					Some(absolute_index as usize)
				}
			})
    }

    pub fn point_at_text_index(&self, index: usize) -> Point {
		let index = index as CFIndex;
		self.text_frame.lines.iter()
			.find(|line| line.char_range.contains(index))
			.map(|line| {
				let origin: Point = line.origin.into();
				let line_index = index - line.char_range.location;
				let offset = line.line.offset_for_string_index(line_index);
				origin + Point::new(offset, 0.0)
			})
			.unwrap_or(Point::ZERO)
	}

	pub fn frame(&self) -> IRef<CTFrame> {
		self.text_frame.frame.clone()
	}

	pub fn as_str(&self) -> &str {
		&self.text
	}
}