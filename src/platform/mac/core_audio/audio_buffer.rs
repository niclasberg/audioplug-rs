use std::os::raw::c_void;

use objc2::{Encode, RefEncode};


#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioBufferList {
	mNumberBuffers: u32,
    mBuffers: *mut AudioBuffer
}

unsafe impl Encode for AudioBufferList {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("AudioBufferList", &[
		<u32>::ENCODING,
		<*mut AudioBuffer>::ENCODING
	]);
}

unsafe impl RefEncode for AudioBufferList {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioBuffer {
	mNumberChannels: u32,
    mDataByteSize: u32,
    mData: *mut c_void
}

unsafe impl Encode for AudioBuffer {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("AudioBuffer", &[
		<u32>::ENCODING,
		<u32>::ENCODING,
		<*mut c_void>::ENCODING
	]);
}

unsafe impl RefEncode for AudioBuffer {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&Self::ENCODING);
}