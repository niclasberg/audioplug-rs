use std::os::raw::c_void;


#[repr(C)]
pub struct AudioBufferList {
	mNumberBuffers: u32,
    mBuffers: *mut AudioBuffer
}

#[repr(C)]
pub struct AudioBuffer {
	mNumberChannels: u32,
    mDataByteSize: u32,
    mData: *mut c_void
}