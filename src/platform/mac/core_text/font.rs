use icrate::Foundation::CGFloat;

use crate::platform::{CFType, mac::{core_foundation::CFString, core_graphics::CGAffineTransform}, IRef};

#[repr(C)]
pub struct CTFont {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFType for CTFont {}

impl CTFont {
	pub fn new(name: &CFString, size: CGFloat, matrix: Option<CGAffineTransform>) -> IRef<Self> {
		unsafe {
			IRef::wrap(CTFontCreateWithName(name, size, matrix.as_ref().map_or_else(|| std::ptr::null(), |x| x as *const _)))
		}
	}
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
	fn CTFontCreateWithName(name: *const CFString, size: CGFloat, matrix: *const CGAffineTransform) -> *const CTFont;

}