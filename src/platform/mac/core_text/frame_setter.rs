use std::mem::MaybeUninit;

use objc2_foundation::CGSize;
use objc2::{Encode, RefEncode};

use crate::platform::mac::{IRef, core_foundation::{CFRange, CFTyped, CFDictionary, CFAttributedString, CFTypeID}, core_graphics::CGPath};

use super::CTFrame;

#[repr(C)]
pub struct CTFrameSetter {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFTyped for CTFrameSetter {
    fn type_id() -> CFTypeID {
        unsafe { CTFramesetterGetTypeID() }
    }
}

unsafe impl Encode for CTFrameSetter {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CTFrameSetter", &[]);
}

unsafe impl RefEncode for CTFrameSetter {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CTFrameSetter::ENCODING);
}

impl CTFrameSetter {
	pub fn from_attributed_string(attributed_string: &CFAttributedString) -> IRef<Self> {
		unsafe { IRef::wrap(CTFramesetterCreateWithAttributedString(attributed_string)) }
	}

	pub fn suggest_frame_size_with_constraints(&self, string_range: CFRange, frame_attributes: Option<&CFDictionary>, constraints: CGSize) -> (CFRange, CGSize) {
		unsafe {
			let mut fit_range = MaybeUninit::<CFRange>::uninit();
			let result = CTFramesetterSuggestFrameSizeWithConstraints(self, string_range, frame_attributes.map(|x| x as *const _).unwrap_or(std::ptr::null()), constraints, fit_range.as_mut_ptr());
			let fit_range = fit_range.assume_init();
			(fit_range, result)
		}
	}

	pub fn create_frame(&self, string_range: CFRange, path: &CGPath, frame_attributes: Option<&CFDictionary>) -> IRef<CTFrame> {
		unsafe {
			let path_ptr = CTFramesetterCreateFrame(self, string_range, path, frame_attributes.map(|x| x as *const _).unwrap_or(std::ptr::null()));
			IRef::wrap(path_ptr)
		}
	}
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
	fn CTFramesetterCreateWithAttributedString(attrString: *const CFAttributedString) -> *mut CTFrameSetter;
	fn CTFramesetterSuggestFrameSizeWithConstraints(framesetter: *const CTFrameSetter, string_range: CFRange, frame_attributes: *const CFDictionary, constraints: CGSize, fit_range: *mut CFRange) -> CGSize;
	fn CTFramesetterCreateFrame(framesetter: *const CTFrameSetter, string_range: CFRange, path: *const CGPath, frame_attributes: *const CFDictionary) -> *const CTFrame;
	fn CTFramesetterGetTypeID() -> CFTypeID;
}