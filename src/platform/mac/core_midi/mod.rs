use super::cf_enum;

pub type MIDITimeStamp = u64;

cf_enum!(
    pub enum MIDIProtocolID: i32 {
        MIDI_1_0 = 1,
        MIDI_2_0 = 2,
    }
);

#[repr(C)]
pub struct MIDIEventList {
    pub protocol: MIDIProtocolID,
    pub num_packets: u32,
    pub packets: *mut MIDIEventPacket,
}

#[repr(C)]
pub struct MIDIEventPacket {
    pub time_stamp: MIDITimeStamp,
    pub word_count: u32,
    pub words: [u32; 64],
}
