use std::cell::RefCell;

use icrate::Foundation::{NSAttributedString, NSString, NSDictionary, CGSize};
use objc2::rc::Id;

use crate::{text::FontWeight, core::Size};

use super::{core_text::CTFrameSetter, IRef, core_foundation::{CFDictionary, CFRange}};

pub struct TextLayout{
    pub(super) attributed_string: Id<NSAttributedString>,
	pub(super) frame_setter: IRef<CTFrameSetter>,
	pub(super) max_size: CGSize,
	suggested_size_result: RefCell<Option<(CFRange, CGSize)>>
}

impl TextLayout {
    pub fn new(
        string: &str, 
        font_family: &str, 
        font_weight: FontWeight,
        font_size: f32,
        max_size: Size
    ) -> Self {
		let string = NSString::from_str(string);
		let attributes = NSDictionary::new();
		let attributed_string = unsafe { NSAttributedString::new_with_attributes(string.as_ref(), attributes.as_ref()) };
		let frame_setter = CTFrameSetter::from_attributed_string(&attributed_string);

        Self {
			attributed_string,
			frame_setter,
			max_size: max_size.into(),
			suggested_size_result: RefCell::new(None)
		}
    }

    pub fn set_max_size(&mut self, size: Size) {
		self.max_size = size.into();
    }

	pub fn measure(&self) -> Size {
		self.suggested_range_and_size().1.into()
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