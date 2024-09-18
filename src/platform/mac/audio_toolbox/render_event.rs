use objc2::{Encode, Encoding, RefEncode};

use super::cf_enum;

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
	event_sample_time: AUEventSampleTime,
	/// The type of the event.
	event_type: AURenderEventType,
	/// Must be 0.
	reserved: u8
}

unsafe impl Encode for AURenderEventHeader {
    const ENCODING: Encoding = Encoding::Struct("AURenderEventHeader", &[]);
}

cf_enum!(
	#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
	pub enum AURenderEventType: u8 {
		AURenderEventParameter		= 1,
		AURenderEventParameterRamp	= 2,
		AURenderEventMIDI			= 8,
		AURenderEventMIDISysEx		= 9,
		AURenderEventMIDIEventList  = 10
	}
);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AUParameterEvent {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	event_sample_time: AUEventSampleTime,
	/// AURenderEventParameter or AURenderEventParameterRamp.
	event_type: AURenderEventType,
	/// Must be 0.
	reserved: [u8; 3],					
	/// If greater than 0, the event is a parameter ramp; should be 0 for a non-ramped event.
	ramp_duration_sample_frames: AUAudioFrameCount,
	/// The parameter to change.								
	parameter_address: AUParameterAddress,
	/// If ramped, the parameter value at the end of the ramp; for a non-ramped event, the new value.
	value: AUValue,				
}

unsafe impl Encode for AUParameterEvent {
    const ENCODING: Encoding = u8::ENCODING;
}

unsafe impl RefEncode for AUParameterEvent {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
/// Describes a single scheduled MIDI event.
pub struct AUMIDIEvent {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	event_sample_time: AUEventSampleTime,
	/// AURenderEventMIDI or AURenderEventMIDISysEx.
	event_type: AURenderEventType,
	/// Must be 0.
	reserved: u8,
	/// The number of valid MIDI bytes in the data field. 1, 2 or 3 for most MIDI events, but can be longer for system-exclusive (sys-ex) events.
	length: u16,
	/// The virtual cable number.
	cable: u8,
	/// The bytes of the MIDI event. Running status will not be used.
	data: [u8; 3]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Describes a single scheduled MIDIEventList.
pub struct AUMIDIEventList {
	/// The next event in a linked list of events.
	next: *mut AURenderEvent,		
	/// The sample time at which the event is scheduled to occur.
	event_sample_time: AUEventSampleTime,
	/// AURenderEventMIDI or AURenderEventMIDISysEx.
	event_type: AURenderEventType,			
	/// Must be 0.
	reserved: u8,			
	/// The virtual cable number.
	cable: u8,				
	// A structure containing UMP packets.
	//eventList: MIDIEventList			
}

#[repr(C)]
pub union AURenderEvent {
	head: AURenderEventHeader,
	parameter: AUParameterEvent,
	midi: AUMIDIEvent,
	midi_events_list: AUMIDIEventList,
}

unsafe impl Encode for AURenderEvent {
    const ENCODING: objc2::Encoding = objc2::Encoding::Union("AURenderEvent", &[
		AURenderEventHeader::ENCODING
	]);
}

unsafe impl RefEncode for AURenderEvent {
	const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&Self::ENCODING);
}