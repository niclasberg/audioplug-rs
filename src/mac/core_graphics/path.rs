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
    unsafe fn release(this: *mut Self) {
        CGPathRelease(this)
    }

    unsafe fn retain(this: *mut Self) {
        CGPathRetain(this)
    }
}

impl CGPath {
	fn new_mut() -> IRef<CGPath> {
		unsafe {

		}
	}
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
	fn CGPathCreateMutable() -> *mut CGPath;
}