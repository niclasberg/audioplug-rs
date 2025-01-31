#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note(pub u8);

fn semitone_freq_ratio() -> f32 {
  2.0_f32.powf(1.0 / 12.0)
}

impl Note {
    pub fn from_midi(note: u8) -> Self {
        Self(note)
    }

    pub fn midi_note(&self) -> u8 {
        self.0
    }

    pub fn frequency_hz(&self) -> f32 {
        440.0 * semitone_freq_ratio().powi(self.0 as i32 - 69)
    }

	pub fn octave_and_semitone(&self) -> (u8, u8) {
		let octave = self.0 / 12;
		(octave, self.0 - octave)
	}
}

pub enum NoteEvent {
	NoteOn {
		channel: i16,
		sample_offset: i32,
		note: Note
	},
	NoteOff {
		channel: i16,
		sample_offset: i32,
		note: Note
	}
}