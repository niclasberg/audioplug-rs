use super::cf_enum;

cf_enum!(
	pub enum MIDIProtocolID: i32 {
		MIDI_1_0 = 1,
		MIDI_2_0 = 2,
	}
);

pub struct MIDIEventList {
	protocol: MIDIProtocolID,
	num_packets: u32,
	//packet: *mut MIDIEventPacket,
}