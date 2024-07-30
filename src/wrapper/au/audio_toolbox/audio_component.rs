use objc2::{Encoding, RefEncode, Encode};


type OSType = u32; // 4CC

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioComponentDescription {
	pub component_type: OSType,
	pub component_sub_type: OSType,
    pub manufacturer: OSType,
    pub componentFlags: u32,
    pub componentFlagsMask: u32
}

unsafe impl Encode for AudioComponentDescription {
    const ENCODING: Encoding = Encoding::Struct("AudioComponentDescription", &[
		<OSType>::ENCODING,
		<OSType>::ENCODING,
		<OSType>::ENCODING,
		<u32>::ENCODING,
		<u32>::ENCODING
	]);
}