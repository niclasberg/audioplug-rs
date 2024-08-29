use objc2::{Encoding, RefEncode, Encode};
use bitflags::bitflags;

type OSType = u32; // 4CC

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AudioComponentFlags(pub u32);
bitflags! {
	impl AudioComponentFlags: u32 {
		/// Called on a render notification Proc - which is called either before or after the render operation of the audio unit. If this flag is set, the proc is being called before the render operation is performed.
		const PreRender = (1 << 2);
		const Unsearchable = 1;
		const SandboxSafe = 2;
		const IsV3AudioUnit = 4;
		const RequiresAsyncInstantiation = 8;
		const CanLoadInProcess = 0x10;
	}
}

unsafe impl Encode for AudioComponentFlags {
    const ENCODING: Encoding = u32::ENCODING;
}

unsafe impl RefEncode for AudioComponentFlags {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioComponentDescription {
	pub component_type: OSType,
	pub component_sub_type: OSType,
    pub manufacturer: OSType,
    pub flags: AudioComponentFlags,
    pub flags_mask: u32
}

unsafe impl Encode for AudioComponentDescription {
    const ENCODING: Encoding = Encoding::Struct("AudioComponentDescription", &[
		<OSType>::ENCODING,
		<OSType>::ENCODING,
		<OSType>::ENCODING,
		<AudioComponentFlags>::ENCODING,
		<u32>::ENCODING
	]);
}