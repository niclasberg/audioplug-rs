use icrate::Foundation::NSAttributedString;
use objc2::{Encode, RefEncode};

use crate::mac::{IRef, IRefCounted, CFType};

#[repr(C)]
pub struct CTFrameSetter {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

unsafe impl CFType for CTFrameSetter {}

unsafe impl Encode for CTFrameSetter {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CTFrameSetter", &[]);
}

unsafe impl RefEncode for CTFrameSetter {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&CTFrameSetter::ENCODING);
}

impl CTFrameSetter {
	pub fn from_attributed_string(attributed_string: &NSAttributedString) -> IRef<Self> {
		unsafe { IRef::wrap(CTFramesetterCreateWithAttributedString(attributed_string)) }
	}
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
	fn CTFramesetterCreateWithAttributedString(attrString: *const NSAttributedString) -> *mut CTFrameSetter;
}