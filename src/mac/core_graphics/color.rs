use crate::mac::{IRefCounted, IRef};

use super::CGFloat;
use objc2::{Encode, RefEncode};

#[repr(C)]
pub struct CGColor {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl Encode for CGColor {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGColor", &[]);
}

unsafe impl RefEncode for CGColor {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CGColor::ENCODING);
}

unsafe impl IRefCounted for CGColor {
    unsafe fn release(this: *mut Self) {
        CGColorRelease(this)
    }

    unsafe fn retain(this: *mut Self) {
        CGColorRetain(this)
    }
}

impl CGColor {
	pub fn from_rgba(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> IRef<Self> {
		unsafe { IRef::wrap(CGColorCreateSRGB(red, green, blue, alpha)) }
	}
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGColorCreateSRGB(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> *mut CGColor;
	fn CGColorRelease(color: *mut CGColor);
	fn CGColorRetain(color: *mut CGColor);
}