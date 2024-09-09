use std::ffi::c_void;

use block2::Block;
use objc2::{extern_class, extern_methods, mutability, rc::Retained, ClassType, Encode, Encoding, RefEncode};
use objc2_foundation::{NSArray, NSInteger, NSNumber, NSObject, NSString};
use c_enum::c_enum;
use bitflags::bitflags;

pub type AUValue = f32;
pub type AUParameterAddress = u64;
pub type AUParameterObserverToken = *mut c_void;

c_enum!(
	pub enum AudioUnitParameterUnit: u32 {
		Generic				= 0,
		Indexed				= 1,
		Boolean				= 2,
		Percent				= 3,
		Seconds				= 4,
		SampleFrames		= 5,
		Phase				= 6,
		Rate				= 7,
		Hertz				= 8,
		Cents				= 9,
		RelativeSemiTones	= 10,
		MIDINoteNumber		= 11,
		MIDIController		= 12,
		Decibels			= 13,
		LinearGain			= 14,
		Degrees				= 15,
		EqualPowerCrossfade = 16,
		MixerFaderCurve1	= 17,
		Pan					= 18,
		Meters				= 19,
		AbsoluteCents		= 20,
		Octaves				= 21,
		BPM					= 22,
		Beats               = 23,
		Milliseconds		= 24,
		Ratio				= 25,
		CustomUnit			= 26,
		MIDI2Controller	 	= 27
	}
);

unsafe impl Encode for AudioUnitParameterUnit {
    const ENCODING: Encoding = u32::ENCODING;
}

unsafe impl RefEncode for AudioUnitParameterUnit {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AudioUnitParameterOptions(pub u32);
bitflags! {
	impl AudioUnitParameterOptions: u32 {
		/// Called on a render notification Proc - which is called either before or after the render operation of the audio unit. If this flag is set, the proc is being called before the render operation is performed.
		const CFNameRelease		= (1 << 4);

		const OmitFromPresets		= (1 << 13);
		const PlotHistory			= (1 << 14);
		const MeterReadOnly		= (1 << 15);
		
		// bit positions 18,17,16 are set aside for display scales. bit 19 is reserved.
		const DisplayMask			= (7 << 16) | (1 << 22);
		const DisplaySquareRoot	= (1 << 16);
		const DisplaySquared		= (2 << 16);
		const DisplayCubed		= (3 << 16);
		const DisplayCubeRoot		= (4 << 16);
		const DisplayExponential	= (5 << 16);

		const HasClump	 		= (1 << 20);
		const ValuesHaveStrings	= (1 << 21);
		
		const DisplayLogarithmic 	= (1 << 22);
		
		const IsHighResolution 	= (1 << 23);
		const NonRealTime 		= (1 << 24);
		const CanRamp 			= (1 << 25);
		const ExpertMode 			= (1 << 26);
		const HasCFNameString 	= (1 << 27);
		const IsGlobalMeta 		= (1 << 28);
		const IsElementMeta		= (1 << 29);
		const IsReadable			= (1 << 30);
		const IsWritable			= 1 << 31;
	}
}

unsafe impl Encode for AudioUnitParameterOptions {
    const ENCODING: Encoding = u32::ENCODING;
}

unsafe impl RefEncode for AudioUnitParameterOptions {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

c_enum!(
	pub enum AUParameterAutomationEventType: u32 {
		/// The event contains an updated value for the parameter.
		Value = 0,
		/// The event marks an initial "touch" gesture on a UI element.
		Touch = 1,
		/// The event marks a final "release" gesture on a UI element.
		Release = 2
	}
);

unsafe impl Encode for AUParameterAutomationEventType {
    const ENCODING: Encoding = u32::ENCODING;
}

unsafe impl RefEncode for AUParameterAutomationEventType {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
/// An event recording the changing of a parameter at a particular host time.
pub struct AURecordedParameterEvent {
	/// The host time at which the event occurred.
	pub host_time: u64,
	/// The address of the parameter whose value changed.
	pub address: AUParameterAddress,		
	/// The value of the parameter at that time.
	pub value: AUValue,
}

unsafe impl Encode for AURecordedParameterEvent {
    const ENCODING: Encoding = Encoding::Struct("AURecordedParameterEvent", &[
		u64::ENCODING,
		AUParameterAddress::ENCODING,
		AUValue::ENCODING
	]);
}

unsafe impl RefEncode for AURecordedParameterEvent {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// An event recording the changing of a parameter, possibly including events such as touch and release gestures, at a particular host time.
#[repr(C)]
pub struct AUParameterAutomationEvent {
	/// The host time at which the event occurred.
	pub host_time: u64,
	/// The address of the parameter whose value changed.
	pub address: AUParameterAddress,
	/// The value of the parameter at that time.
	pub value: AUValue,
	/// The type of the event.
	pub event_type: AUParameterAutomationEventType,
	/// Reserved
	pub reserved: u64
}

unsafe impl Encode for AUParameterAutomationEvent {
    const ENCODING: Encoding = Encoding::Struct("AUParameterAutomationEvent", &[
		u64::ENCODING,
		AUParameterAddress::ENCODING,
		AUValue::ENCODING,
		AUParameterAutomationEventType::ENCODING,
		u64::ENCODING
	]);
}

unsafe impl RefEncode for AUParameterAutomationEvent {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AUParameterNode;

	unsafe impl ClassType for AUParameterNode {
		type Super = NSObject;
		type Mutability = mutability::InteriorMutable;
	}
);

/// A block called to signal that the value of a parameter has changed.
pub type AUParameterObserver = Block<dyn Fn(AUParameterAddress, AUValue)>;

/// A block called to record parameter changes as automation events.
pub type AUParameterRecordingObserver = Block<dyn Fn(NSInteger, *const AURecordedParameterEvent)>;

/// A block called to record parameter changes as automation events.
pub type AUParameterAutomationObserver = Block<dyn Fn(NSInteger, *const AUParameterAutomationEvent)>;

/// A block called to notify the audio unit implementation of changes to a parameter value.
pub type AUImplementorValueObserver = Block<dyn Fn(*mut AUParameter, AUValue)>;

/// A block called to fetch a parameter’s current value from the audio unit implementation.
pub type AUImplementorValueProvider = Block<dyn Fn(*mut AUParameter) -> AUValue>;

/// A block called to convert a parameter value to a string representation.
pub type AUImplementorStringFromValueCallback = Block<dyn Fn(*mut AUParameter, *const AUValue) -> *mut NSString>;

/// A block called to convert a string to a parameter value.
pub type AUImplementorValueFromStringCallback = Block<dyn Fn(*mut AUParameter, *mut NSString) -> AUValue>;

extern_methods!(
	unsafe impl AUParameterNode {
		/// A non-localized, permanent name for the parameter node.
		#[method_id(identifier)]
		pub fn identifier(&self) -> Retained<NSString>;

		/// A key path generated by concatenating the identifiers of the parameter and its parents.
		#[method_id(keyPath)]
		#[allow(non_snake_case)]
		pub fn keyPath(&self) -> Retained<NSString>;

		#[method_id(displayName)]
		#[allow(non_snake_case)]
		pub fn displayName(&self) -> Retained<NSString>;
		
		#[method_id(displayNameWithLength:)]
		#[allow(non_snake_case)]
		pub fn displayNameWithLength(&self, maximumLength: NSInteger) -> Retained<NSString>;

		#[method(tokenByAddingParameterObserver:)]
		#[allow(non_snake_case)]
		pub fn tokenByAddingParameterObserver(&self, observer: &AUParameterObserver) -> AUParameterObserverToken;

		#[method(tokenByAddingParameterRecordingObserver:)]
		#[allow(non_snake_case)]
		pub fn tokenByAddingParameterRecordingObserver(&self, observer: &AUParameterRecordingObserver) -> AUParameterObserverToken;

		#[method(tokenByAddingParameterAutomationObserver:)]
		#[allow(non_snake_case)]
		pub fn tokenByAddingParameterAutomationObserver(&self, observer: &AUParameterAutomationObserver) -> AUParameterObserverToken;

		#[method(removeParameterObserver:)]
		#[allow(non_snake_case)]
		pub fn removeParameterObserver(&self, token: AUParameterObserverToken);
	}
);

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AUParameterGroup;

	unsafe impl ClassType for AUParameterGroup {
		type Super = AUParameterNode;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AUParameterTree;

	unsafe impl ClassType for AUParameterTree {
		type Super = AUParameterGroup;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUParameterTree {
		#[method_id(createParameterWithIdentifier:name:address:min:max:unit:unitName:flags:valueStrings:dependentParameters:)]
		#[allow(non_snake_case)]
		pub fn createParameter(
			identifier: &NSString, 
			name: &NSString, 
			address: AUParameterAddress, 
			min: AUValue, 
			max: AUValue, 
			unit: AudioUnitParameterUnit, 
			unitName: &NSString,
			flags: AudioUnitParameterOptions, 
			valueStrings: &NSArray<NSString>, 
			dependentParameters: &NSArray<NSNumber>
		) -> Retained<AUParameter>;


		#[method_id(createTreeWithChildren:)]
		#[allow(non_snake_case)]
		pub fn createTreeWithChildren(children: &NSArray<AUParameterNode>) -> Retained<Self>;
		
		#[method_id(parameterWithAddress:)]
		#[allow(non_snake_case)]
		pub fn parameterWithAddress(&self, address: AUParameterAddress) -> Option<Retained<AUParameter>>;

		/// Get the callback for parameter value changes.
		#[method(implementorValueObserver)]
		#[allow(non_snake_case)]
		pub fn implementorValueObserver(&self) -> &AUImplementorValueObserver;

		/// Set the callback for parameter value changes.
		#[method(setImplementorValueObserver:)]
		#[allow(non_snake_case)]
		pub fn setImplementorValueObserver(&self, valueObserver: &AUImplementorValueObserver);

		#[method(implementorValueProvider)]
		#[allow(non_snake_case)]
		pub fn implementorValueProvider(&self) -> &AUImplementorValueProvider;

		/// Set the callback for parameter value changes.
		#[method(setImplementorValueProvider:)]
		#[allow(non_snake_case)]
		pub fn setImplementorValueProvider(&self, valueProvider: &AUImplementorValueProvider);
	}
);

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AUParameter;

	unsafe impl ClassType for AUParameter {
		type Super = AUParameterNode;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUParameter {
		#[method(minValue)]
		#[allow(non_snake_case)]
		pub fn minValue(&self) -> AUValue;

		#[method(maxValue)]
		#[allow(non_snake_case)]
		pub fn maxValue(&self) -> AUValue;

		#[method(unit)]
		pub fn unit(&self) -> AudioUnitParameterUnit;

		#[method(flags)]
		pub fn flags(&self) -> AudioUnitParameterOptions;

		#[method(address)]
		pub fn address(&self) -> AUParameterAddress;

		#[method(value)]
		pub fn value(&self) -> AUValue;

		#[method(setValue:originator:)]
		#[allow(non_snake_case)]
		pub fn setValue_orginator(&self, value: AUValue, originator: AUParameterObserverToken);

		#[method(setValue:originator:atHostTime:)]
		#[allow(non_snake_case)]
		pub fn setValue_orginator_atHostTime(&self, value: AUValue, originator: AUParameterObserverToken, host_time: u64);

		#[method(setValue:originator:atHostTime:eventType:)]
		#[allow(non_snake_case)]
		pub fn setValue_orginator_atHostTime_eventType(&self, value: AUValue, originator: AUParameterObserverToken, host_time: u64, event_type: AUParameterAutomationEventType);
	}
);