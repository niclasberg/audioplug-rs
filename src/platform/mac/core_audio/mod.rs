use std::os::raw::c_void;

mod timestamp;
use c_enum::c_enum;
pub use timestamp::*;
mod audio_device;
mod properties;
mod audio_stream;
mod audio_system_object;
mod audio_object;
mod error;
mod audio_buffer;
pub use audio_buffer::*;

pub type OSStatus = i32;

pub type AudioClassID = u32;
pub type AudioObjectID = u32;
pub type AudioObjectPropertyElement = u32;
pub type AudioObjectPropertySelector = u32;
pub type AudioHardwarePropertyID = AudioObjectPropertySelector;
pub type AudioDeviceID = AudioObjectID;
pub type AudioStreamID = AudioObjectID;
pub type AudioFormatID = u32;


/*#[repr(C)]
struct AudioDriverPlugInHostInfo {
	pub mDeviceID: AudioDeviceID,
	pub mDevicePropertyChangedProc: AudioDriverPlugInDevicePropertyChangedProc,

}*/

c_enum!(
	#[repr(transparent)]
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum AudioObjectPropertyScope: u32 {
		Global         = 1735159650, //'glob'
		Input          = 1768845428, //'inpt'
		Output         = 1869968496, //'outp'
		PlayThrough    = 1886679669 //'ptru'
	}
);

#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioObjectPropertyAddress {
	mElement: AudioObjectPropertyElement,
	mScope: AudioObjectPropertyScope,
	mSelector: AudioObjectPropertySelector
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioBuffer {
	mNumberChannels: u32,
	mDataByteSize: u32,
	mData: *mut c_void
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioBufferList {
	mNumberBuffers: u32,
	mBuffers: *mut AudioBuffer
}

#[repr(C)]
#[allow(non_snake_case)]
struct AudioValueRange {
    mMinimum: f64,
    mMaximum: f64
}

c_enum!(
	#[repr(transparent)]
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum AudioFormatFlags: u32 {
		IsFloat                     = (1 << 0),
		IsBigEndian                 = (1 << 1),
		IsSignedInteger             = (1 << 2),
		IsPacked                    = (1 << 3),
		IsAlignedHigh               = (1 << 4),
		IsNonInterleaved            = (1 << 5),
		IsNonMixable                = (1 << 6),
		AreAllClear                = 0x80000000,
		LinearPCMSampleFractionShift    = 7,
		kLinearPCMSampleFractionMask     = (0x3F << 7),
		AppleLossless16BitSourceData    = 1,
		AppleLossless20BitSourceData    = 2,
		AppleLossless24BitSourceData    = 3,
		AppleLossless32BitSourceData    = 4
	}
);

#[repr(C)]
#[allow(non_snake_case)]
struct AudioStreamBasicDescription {
    mSampleRate: f64,
    mFormatID: AudioFormatID,
    mFormatFlags: AudioFormatFlags,
    mBytesPerPacket: u32,
    mFramesPerPacket: u32,
    mBytesPerFrame: u32,
    mChannelsPerFrame: u32,
    mBitsPerChannel: u32,
    mReserved: u32
}

// typedef void (^AudioObjectPropertyListenerBlock)(UInt32 inNumberAddresses, const AudioObjectPropertyAddress *inAddresses);
pub type AudioObjectPropertyListenerBlock = unsafe extern "C" fn(u32, *const AudioObjectPropertyAddress);
// typedef OSStatus (*AudioObjectPropertyListenerProc)(AudioObjectID inObjectID, UInt32 inNumberAddresses, const AudioObjectPropertyAddress *inAddresses, void *inClientData);
pub type AudioObjectPropertyListenerProc = unsafe extern "C" fn(AudioObjectID, u32, *const AudioObjectPropertyAddress, *mut c_void) -> OSStatus;
// typedef OSStatus (*AudioHardwarePropertyListenerProc)(AudioHardwarePropertyID inPropertyID, void *inClientData);
pub type AudioHardwarePropertyListenerProc = unsafe extern "C" fn(AudioHardwarePropertyID, *mut c_void);

//type AudioDeviceIOBlock = unsafe extern "C" fn(i32, *mut c_void) (*const AudioTimeStamp, const AudioBufferList *inInputData, const AudioTimeStamp *inInputTime, AudioBufferList *outOutputData, const AudioTimeStamp *inOutputTime);

#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
	fn AudioConvertHostTimeToNanos(inHostTime: u64) -> u64;
	fn AudioConvertNanosToHostTime(inNanos: u64) -> u64;

	fn AudioObjectGetPropertyData(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress, inQualifierDataSize: u32, inQualifierData: *const c_void, ioDataSize: *mut u32, outData: *mut c_void	) -> OSStatus;
	fn AudioObjectGetPropertyDataSize(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress, inQualifierDataSize: u32, inQualifierData: *const c_void, outDataSize: *mut u32) -> OSStatus;
	fn AudioObjectHasProperty(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress) -> bool;
	fn AudioObjectIsPropertySettable(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress, outIsSettable: *mut bool) -> OSStatus;
	fn AudioObjectAddPropertyListener(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress, inListener: AudioObjectPropertyListenerProc, inClientData: *mut c_void) -> OSStatus;
	fn AudioObjectRemovePropertyListener(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress, inListener: AudioObjectPropertyListenerProc, inClientData: *mut c_void) -> OSStatus;
	fn AudioObjectSetPropertyData(inObjectID: AudioObjectID, inAddress: *const AudioObjectPropertyAddress, inQualifierDataSize: u32, inQualifierData: *const c_void, inDataSize: u32, inData: *const c_void) -> OSStatus;
}