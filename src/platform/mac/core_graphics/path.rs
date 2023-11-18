use icrate::Foundation::CGRect;
use objc2::{Encode, RefEncode};

use crate::platform::mac::{IRefCounted, IRef};

use super::CGAffineTransform;

#[repr(C)]
pub struct CGPath {
	_data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl Encode for CGPath {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGPath", &[]);
}

unsafe impl RefEncode for CGPath {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CGPath::ENCODING);
}

unsafe impl IRefCounted for CGPath {
    unsafe fn release(this: *const Self) {
        CGPathRelease(this)
    }

    unsafe fn retain(this: *const Self) {
        CGPathRetain(this);
    }
}

impl CGPath {
	fn new_mut() -> IRef<CGPath> {
		todo!()
	}

	pub fn create_with_rect(rect: CGRect, transform: Option<&CGAffineTransform>) -> IRef<CGPath> {
		unsafe {
			let ptr = CGPathCreateWithRect(rect, transform.map_or_else(|| std::ptr::null(), |x| x as *const _));
			IRef::wrap_and_retain(ptr)
		}
	}
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGPathCreateMutable() -> *mut CGPath;
	fn CGPathRelease(path: *const CGPath);
	fn CGPathRetain(path: *const CGPath) -> *const CGPath;
	fn CGPathCreateWithRect(rect: CGRect, transform: *const CGAffineTransform) -> *const CGPath;
}