use objc2_foundation::{CGPoint, CGFloat};

use crate::platform::mac::core_foundation::{CFTyped, CFTypeID, CFIndex, CFRange};

#[repr(C)]
pub struct CTLine {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFTyped for CTLine {
    fn type_id() -> CFTypeID {
        unsafe { CTLineGetTypeID() }
    }
}

impl CTLine {
	pub fn string_index_for_position(&self, position: CGPoint) -> CFIndex {
		unsafe {
			CTLineGetStringIndexForPosition(self, position)
		}
	}

	pub fn offset_for_string_index(&self, index: CFIndex) -> CGFloat {
		unsafe {
			CTLineGetOffsetForStringIndex(self, index, std::ptr::null_mut())
		}
	}

	pub fn string_range(&self) -> CFRange {
		unsafe { CTLineGetStringRange(self) }
	}
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
	fn CTLineGetTypeID() -> CFTypeID;
	fn CTLineGetStringIndexForPosition(line: *const CTLine, position: CGPoint) -> CFIndex;
	fn CTLineGetOffsetForStringIndex(line: *const CTLine, charIndex: CFIndex, secondaryOffset: *mut CGFloat) -> CGFloat;
	fn CTLineGetStringRange(line: *const CTLine) -> CFRange;

}