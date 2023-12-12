use icrate::Foundation::CGPoint;
use objc2::{Encode, RefEncode};

use crate::platform::mac::{core_graphics::{CGContext, CGPath}, IRef, core_foundation::{CFTypeID, CFArray, CFIndex}};
use crate::platform::mac::core_foundation::{CFRange, CFTyped};

use super::CTLine;

#[repr(C)]
pub struct CTFrame {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFTyped for CTFrame {
    fn type_id() -> CFTypeID {
        unsafe { CTFrameGetTypeID() }
    }
}

unsafe impl Encode for CTFrame {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CTFrame", &[]);
}

unsafe impl RefEncode for CTFrame {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CTFrame::ENCODING);
}

#[allow(dead_code)]
impl CTFrame {
	pub fn draw(&self, context: &CGContext) {
		unsafe {
			CTFrameDraw(self, context);
		}
	}

	pub fn path(&self) -> IRef<CGPath> {
		unsafe { 
			IRef::wrap_and_retain(CTFrameGetPath(self))
		}
	}

	pub fn get_visible_string_range(&self) -> CFRange {
		unsafe {
			CTFrameGetVisibleStringRange(self)
		}
	}

	pub fn get_lines(&self) -> Vec<IRef<CTLine>> {
		unsafe {
			let lines = CTFrameGetLines(self);
			(&*lines).as_vec_of()
		}
	}

	pub fn lines_count(&self) -> CFIndex {
		unsafe { &*(CTFrameGetLines(self)) }.get_count()
	}

	pub fn get_line_origins(&self) -> Vec<CGPoint> {
		let count = self.lines_count();
		let mut origins = Vec::with_capacity(count as usize);
		unsafe {
			CTFrameGetLineOrigins(self, CFRange { location: 0, length: count }, origins.as_mut_ptr());
			origins.set_len(count as usize);
		}
		origins
	}
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
	fn CTFrameDraw(frame: *const CTFrame, context: *const CGContext);
	fn CTFrameGetPath(frame: *const CTFrame) -> *const CGPath;
	fn CTFrameGetVisibleStringRange(frame: *const CTFrame) -> CFRange;
	fn CTFrameGetTypeID() -> CFTypeID;
	fn CTFrameGetLines(frame: *const CTFrame) -> *const CFArray;
	fn CTFrameGetLineOrigins(frame: *const CTFrame, range: CFRange, origins: *const CGPoint);
}