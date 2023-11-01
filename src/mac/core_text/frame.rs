use objc2::{Encode, RefEncode};

use crate::mac::{CFType, core_graphics::CGContext};

#[repr(C)]
pub struct CTFrame {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFType for CTFrame {}

unsafe impl Encode for CTFrame {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CTFrame", &[]);
}

unsafe impl RefEncode for CTFrame {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CTFrame::ENCODING);
}

impl CTFrame {
	fn draw(&self, context: &CGContext) {
		unsafe {
			CTFrameDraw(self, context);
		}
	}
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
	fn CTFrameDraw(frame: *const CTFrame, context: *const CGContext);
}