use c_enum::c_enum;
use objc2::{extern_class, extern_methods, rc::{Allocated, Retained}, ClassType, Encode, Encoding, RefEncode};
use objc2_foundation::{NSArray, NSError, NSInteger, NSObject};

use super::AUAudioUnit;

pub type AVAudioChannelCount = u32;

c_enum!(
	pub enum AUAudioUnitBusType: NSInteger {
		AUAudioUnitBusTypeInput		= 1,
		AUAudioUnitBusTypeOutput	= 2
	}
);

unsafe impl Encode for AUAudioUnitBusType {
    const ENCODING: Encoding = NSInteger::ENCODING;
}

unsafe impl RefEncode for AUAudioUnitBusType {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

extern_class!(
	pub struct AUAudioUnitBusArray;

	unsafe impl ClassType for AUAudioUnitBusArray {
		type Super = NSObject;
		type Mutability = objc2::mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUAudioUnitBusArray {
		#[method_id(initWithAudioUnit:busType:busses:)]
		#[allow(non_snake_case)]
		pub unsafe fn initWithAudioUnit_busType_busses(
			this: Allocated<Self>,
			owner: *mut AUAudioUnit,
			bus_type: AUAudioUnitBusType,
			busses: &NSArray<AUAudioUnitBus>
		) -> Retained<Self>;

		#[method_id(initWithAudioUnit:busType:)]
		#[allow(non_snake_case)]
		pub unsafe fn initWithAudioUnit_busType(
			this: Allocated<Self>,
			owner: *mut AUAudioUnit,
			bus_type: AUAudioUnitBusType) -> Retained<Self>;
	}
);

//	@method		initWithAudioUnit:busType:
//	@brief		Initializes an empty bus array.
//- (instancetype)initWithAudioUnit:(AUAudioUnit *)owner busType:(AUAudioUnitBusType)busType;
//@property (NS_NONATOMIC_IOSONLY, readonly) NSUInteger count;

extern_class!(
	pub struct AUAudioUnitBus;

	unsafe impl ClassType for AUAudioUnitBus {
		type Super = NSObject;
		type Mutability = objc2::mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUAudioUnitBus {
		#[method_id(initWithFormat:error:)]
		#[allow(non_snake_case)]
		pub unsafe fn initWithFormat_error(
			this: Allocated<Self>,
			format: &AVAudioFormat,
			error: *mut *mut NSError) -> Retained<Self>;
	}
);

extern_class!(
	pub struct AVAudioFormat;

	unsafe impl ClassType for AVAudioFormat {
		type Super = NSObject;
		type Mutability = objc2::mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AVAudioFormat {
		#[method_id(initStandardFormatWithSampleRate:channels:)]
		#[allow(non_snake_case)]
		pub unsafe fn initStandardFormatWithSampleRate_channels(
			this: Allocated<Self>,
			sampleRate: f64,
			channels: AVAudioChannelCount) -> Retained<Self>;
	}
);