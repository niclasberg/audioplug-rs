use objc2::{extern_class, extern_methods, rc::{Allocated, Retained}};
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
	#[unsafe(super(NSObject))]
	pub struct AUAudioUnitBusArray;
);

impl AUAudioUnitBusArray {
	extern_methods!(
		#[unsafe(method(initWithAudioUnit:busType:busses:))]
		#[allow(non_snake_case)]
		pub unsafe fn initWithAudioUnit_busType_busses(
			this: Allocated<Self>,
			owner: &AUAudioUnit,
			bus_type: AUAudioUnitBusType,
			busses: &NSArray<AUAudioUnitBus>
		) -> Retained<Self>;

		#[unsafe(method(initWithAudioUnit:busType:))]
		#[allow(non_snake_case)]
		pub unsafe fn initWithAudioUnit_busType(
			this: Allocated<Self>,
			owner: &AUAudioUnit,
			bus_type: AUAudioUnitBusType) -> Retained<Self>;

		#[unsafe(method(count))]
		pub fn count(&self) -> NSUInteger;
	);
}
	
extern_class!(
	#[unsafe(super(NSObject))]
	pub struct AUAudioUnitBus;
);

impl AUAudioUnitBus {
	extern_methods!(
		#[unsafe(method(initWithFormat:error:_))]
		#[allow(non_snake_case)]
		pub unsafe fn initWithFormat_error(
			this: Allocated<Self>,
			format: &AVAudioFormat) -> Result<Retained<Self>, Retained<NSError>>;

		#[unsafe(method(maximumChannelCount))]
		#[allow(non_snake_case)]
		pub fn maximumChannelCount(&self) -> AUAudioChannelCount;

		#[unsafe(method(setMaximumChannelCount:))]
		#[allow(non_snake_case)]
		pub fn setMaximumChannelCount(&self, value: AUAudioChannelCount);
	);
}
