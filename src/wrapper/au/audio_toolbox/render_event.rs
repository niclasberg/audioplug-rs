use c_enum::c_enum;
use objc2::{Encode, RefEncode};

type AUEventSampleTime = i64;
type AUParameterAddress = u64;
type AUValue = f32;
type AUAudioFrameCount = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
///	Common header for an AURenderEvent.
pub struct AURenderEventHeader {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	eventSampleTime: AUEventSampleTime,
	/// The type of the event.
	eventType: AURenderEventType,
	/// Must be 0.
	reserved: u8
}

c_enum!(
	#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
	pub enum AURenderEventType: u8 {
		AURenderEventParameter		= 1,
		AURenderEventParameterRamp	= 2,
		AURenderEventMIDI			= 8,
		AURenderEventMIDISysEx		= 9,
		AURenderEventMIDIEventList  = 10
	}
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AUParameterEvent {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	eventSampleTime: AUEventSampleTime,
	/// AURenderEventParameter or AURenderEventParameterRamp.
	eventType: AURenderEventType,
	/// Must be 0.
	reserved: [u8; 3],					
	/// If greater than 0, the event is a parameter ramp; should be 0 for a non-ramped event.
	rampDurationSampleFrames: AUAudioFrameCount,
	/// The parameter to change.								
	parameterAddress: AUParameterAddress,
	/// If ramped, the parameter value at the end of the ramp; for a non-ramped event, the new value.
	value: AUValue,				
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Describes a single scheduled MIDI event.
pub struct AUMIDIEvent {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	eventSampleTime: AUEventSampleTime,
	/// AURenderEventMIDI or AURenderEventMIDISysEx.
	eventType: AURenderEventType,
	/// Must be 0.
	reserved: u8,
	/// The number of valid MIDI bytes in the data field. 1, 2 or 3 for most MIDI events, but can be longer for system-exclusive (sys-ex) events.
	length: u16,
	/// The virtual cable number.
	cable: u8,
	/// The bytes of the MIDI event. Running status will not be used.
	data: [u8; 3]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Describes a single scheduled MIDIEventList.
pub struct AUMIDIEventList {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	eventSampleTime: AUEventSampleTime,
	/// AURenderEventMIDI or AURenderEventMIDISysEx.
	eventType: AURenderEventType,			
	/// Must be 0.
	reserved: u8,			
	/// The virtual cable number.
	cable: u8,				
	// A structure containing UMP packets.
	//eventList: MIDIEventList			
}

pub union AURenderEvent {
	head: AURenderEventHeader,
	parameter: AUParameterEvent,
	MIDI: AUMIDIEvent,
	MIDIEventsList: AUMIDIEventList,
}

unsafe impl Encode for AURenderEvent {
    const ENCODING: objc2::Encoding = objc2::Encoding::Union("AURenderEvent", &[

	]);
}

unsafe impl RefEncode for AURenderEvent {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&Self::ENCODING);
}