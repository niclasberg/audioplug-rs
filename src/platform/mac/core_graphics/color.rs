use crate::{platform::{mac::{IRefCounted, IRef}, CFType}, core::Color};

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

unsafe impl CFType for CGColor { }

impl CGColor {
	pub fn from_rgba(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> IRef<Self> {
		unsafe { IRef::wrap(CGColorCreateSRGB(red, green, blue, alpha)) }
	}

	pub fn from_color(color: Color) -> IRef<Self> {
		Self::from_rgba(color.r.into(), color.g.into(), color.b.into(), color.a.into())
	}
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGColorCreateSRGB(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> *mut CGColor;
	fn CGColorRelease(color: *const CGColor);
	fn CGColorRetain(color: *const CGColor);
}