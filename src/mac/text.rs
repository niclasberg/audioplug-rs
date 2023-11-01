use icrate::Foundation::{NSAttributedString, NSString, NSDictionary};
use objc2::rc::Id;

use crate::{text::FontWeight, core::Size};

use super::{core_text::CTFrameSetter, IRef};

pub struct TextLayout{
    pub(super) attributed_string: Id<NSAttributedString>,
	pub(super) frame_setter: IRef<CTFrameSetter>
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
			frame_setter
		}
    }

    pub fn set_max_size(&mut self, size: Size) {
        
    }

    pub fn measure(&self) -> Size {
        Size::ZERO
    }
}