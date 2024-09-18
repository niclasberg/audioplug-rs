pub enum NoteEvent {
	NoteOn {
		channel: i16,
		sample_offset: i32,
		pitch: i16
	},
	NoteOff {
		channel: i16,
		sample_offset: i32,
		pitch: i16
	}
}