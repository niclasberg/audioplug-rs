use objc2::{extern_class, extern_methods, rc::{Allocated, Retained}, ClassType, Encode, Encoding, RefEncode};
use objc2_foundation::{NSArray, NSError, NSInteger, NSObject, NSUInteger};

use crate::platform::mac::av_foundation::AVAudioFormat;

use super::{cf_enum, AUAudioUnit};

pub type AUAudioChannelCount = u32;

cf_enum!(
	pub enum AUAudioUnitBusType: NSInteger {
		Input		= 1,
		Output	= 2
	}
);

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

		#[method(count)]
		pub fn count(&self) -> NSUInteger;
	}
);

extern_class!(
	pub struct AUAudioUnitBus;

	unsafe impl ClassType for AUAudioUnitBus {
		type Super = NSObject;
		type Mutability = objc2::mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUAudioUnitBus {
		#[method_id(initWithFormat:error:_)]
		#[allow(non_snake_case)]
		pub unsafe fn initWithFormat_error(
			this: Allocated<Self>,
			format: &AVAudioFormat) -> Result<Retained<Self>, Retained<NSError>>;

		#[method(maximumChannelCount)]
		#[allow(non_snake_case)]
		pub fn maximumChannelCount(&self) -> AUAudioChannelCount;

		#[method(setMaximumChannelCount:)]
		#[allow(non_snake_case)]
		pub fn setMaximumChannelCount(&self, value: AUAudioChannelCount);
	}
);

