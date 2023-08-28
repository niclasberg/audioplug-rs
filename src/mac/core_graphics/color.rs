use std::ops::Deref;

use super::CGFloat;
use objc2::{Encode, RefEncode};

pub unsafe trait CGReffable {
	unsafe fn release(this: *mut Self);
	unsafe fn retain(this: *mut Self);
}

pub struct CGRef<T: CGReffable> {
	ptr: *mut T
}

impl<T: CGReffable> CGRef<T> {
	unsafe fn wrap(ptr: *mut T) -> CGRef<T> {
		CGRef { ptr }
	}
}

impl<T: CGReffable> Drop for CGRef<T> {
    fn drop(&mut self) {
		unsafe { <T as CGReffable>::release(self.ptr) };
    }
}

impl<T:CGReffable> Deref for CGRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

#[repr(C)]
pub struct CGColor {
	_data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl Encode for CGColor {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGColor", &[]);
}

unsafe impl RefEncode for CGColor {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CGColor::ENCODING);
}

unsafe impl CGReffable for CGColor {
    unsafe fn release(this: *mut Self) {
        CGColorRelease(this)
    }

    unsafe fn retain(this: *mut Self) {
        CGColorRetain(this)
    }
}

impl CGColor {
	pub fn from_rgba(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> CGRef<Self> {
		unsafe { CGRef::wrap(CGColorCreateSRGB(red, green, blue, alpha)) }
	}
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGColorCreateSRGB(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> *mut CGColor;
	fn CGColorRelease(color: *mut CGColor);
	fn CGColorRetain(color: *mut CGColor);
}