use std::cell::RefCell;

use icrate::Foundation::{NSAttributedString, NSString, NSDictionary, CGSize};
use objc2::rc::Id;

use crate::{text::FontWeight, core::{Color, Size, Point}};

use super::{core_text::{CTFrameSetter, CTFrame, AttributedStringBuilder}, IRef, core_foundation::{CFRange, CFString, CFAttributedString}, IMut, core_graphics::CGColor};

pub struct TextLayout{
    pub(super) attributed_string: IMut<CFAttributedString>,
	pub(super) frame_setter: IRef<CTFrameSetter>,
	pub(super) max_size: CGSize,
	frame: Option<IRef<CTFrame>>,
	suggested_size_result: RefCell<Option<(CFRange, CGSize)>>
}

impl TextLayout {
    pub fn new(
        string: &str, 
        font_family: &str, 
        font_weight: FontWeight,
        font_size: f32,
        max_size: Size,
		color: Color
    ) -> Self {
		let string = CFString::new(string);
		let mut builder = AttributedStringBuilder::new(&string);
		let color = CGColor::from_color(color);
		builder.set_foreground_color(builder.range(), &color);
		let attributed_string = builder.0;
		let frame_setter = CTFrameSetter::from_attributed_string(&attributed_string);

        Self {
			attributed_string,
			frame_setter,
			max_size: max_size.into(),
			frame: None,
			suggested_size_result: RefCell::new(None)
		}
    }

    pub fn set_max_size(&mut self, size: Size) {
		self.max_size = size.into();
    }

	pub fn measure(&self) -> Size {
		self.suggested_range_and_size().1.into()
	}

	pub fn text_index_at_point(&self, point: Point) -> Option<usize> {
        None
    }

    pub fn point_at_text_index(&self, index: usize) -> Option<Point> {
		None
	}

	pub(super) fn suggested_range_and_size(&self) -> (CFRange, CGSize) {
		let mut suggested_size_result = self.suggested_size_result.borrow_mut();
		let range_size = suggested_size_result.get_or_insert_with(|| {
			let string_range = (0..self.attributed_string.length() as isize).into();
			self.frame_setter.suggest_frame_size_with_constraints(string_range, None, self.max_size.into())
		});
		*range_size
	}
}