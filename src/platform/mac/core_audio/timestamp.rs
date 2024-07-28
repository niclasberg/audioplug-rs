use c_enum::c_enum;
use objc2::{Encode, RefEncode};

c_enum!(
	#[repr(transparent)]
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum AudioTimeStampFlags: u32 {
		kAudioTimeStampNothingValid         = 0,
		kAudioTimeStampSampleTimeValid      = (1 << 0),
		kAudioTimeStampHostTimeValid        = (1 << 1),
		kAudioTimeStampRateScalarValid      = (1 << 2),
		kAudioTimeStampWordClockTimeValid   = (1 << 3),
		kAudioTimeStampSMPTETimeValid       = (1 << 4),
		kAudioTimeStampSampleHostTimeValid  = ((1 << 0) | (1 << 1))
	}
);

c_enum!(
	#[repr(transparent)]
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum SMPTETimeType: u32 {
		kSMPTETimeType24        = 0,
		kSMPTETimeType25        = 1,
		kSMPTETimeType30Drop    = 2,
		kSMPTETimeType30        = 3,
		kSMPTETimeType2997      = 4,
		kSMPTETimeType2997Drop  = 5,
		kSMPTETimeType60        = 6,
		kSMPTETimeType5994      = 7,
		kSMPTETimeType60Drop    = 8,
		kSMPTETimeType5994Drop  = 9,
		kSMPTETimeType50        = 10,
		kSMPTETimeType2398      = 11
	}
);

c_enum!(
	#[repr(transparent)]
	#[derive(Copy, Clone, PartialEq, Eq, Hash)]
	pub enum SMPTETimeFlags: u32 {
		kSMPTETimeUnknown   = 0,
		kSMPTETimeValid     = (1 << 0),
		kSMPTETimeRunning   = (1 << 1)
	}
);

#[repr(C)]
#[allow(non_snake_case)]
pub struct SMPTETime {
    mSubframes: i16,
    mSubframeDivisor: i16,
    mCounter: u32,
    mType: SMPTETimeType,
    mFlags: SMPTETimeFlags,
    mHours: i16,
    mMinutes: i16,
    mSeconds: i16,
    mFrames: i16,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioTimeStamp {
	mSampleTime: f64,
    mHostTime: u64,
    mRateScalar: f64,
    mWordClockTime: u64,
    mSMPTETime: SMPTETime,
    mFlags: AudioTimeStampFlags,
    mReserved: u32,
}

unsafe impl Encode for AudioTimeStamp {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("AudioTimeStamp", &[]);
}

unsafe impl RefEncode for AudioTimeStamp {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&Self::ENCODING);
}