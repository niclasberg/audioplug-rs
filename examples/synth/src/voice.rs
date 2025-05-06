use audioplug::{
    dsp::{ADSREnvelope, ADSRParameters},
    midi::Note,
};

pub struct Voice {
    pub note: Note,
    pub ang_freq: f32,
    pub t: f32,
    pub envelope: ADSREnvelope<f32>,
}

impl Voice {
    pub fn new(sample_rate: f32, env_parameters: ADSRParameters<f32>) -> Self {
        Self {
            note: Note::from_midi(0),
            t: 0.0,
            ang_freq: 0.0,
            envelope: ADSREnvelope::new(sample_rate, env_parameters),
        }
    }

    pub fn note_on(&mut self, note: Note) {
        use std::f32::consts::TAU;
        self.ang_freq = note.frequency_hz() * TAU;
        self.note = note;
        self.envelope.note_on();
        self.t = 0.0;
    }
}
