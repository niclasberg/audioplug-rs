use audioplug::{param::{FloatParameter, FloatRange, GroupId, IntParameter, ParameterGroup, ParameterId, Params}, params};

params!(
	pub struct OscillatorParams {
		pub octave: IntParameter,
		pub semitones: IntParameter
	}
);

impl OscillatorParams {
	pub fn new(offset: u32) -> Self {
		Self {
			octave: IntParameter::new(ParameterId::new(offset + 0), "Octave")
				.with_range(-3..=3),
			semitones: IntParameter::new(ParameterId::new(offset + 1), "Semitones")
				.with_range(-11..=11)
		}
	}
}

params!(
    pub struct SynthParams {
        pub amplitude: FloatParameter,
		pub oscillator1: ParameterGroup<OscillatorParams>,
		pub oscillator2: ParameterGroup<OscillatorParams>
    }
);

impl Params for SynthParams {
    fn new() -> Self {
        Self {  
            amplitude: FloatParameter::new(ParameterId::new(1), "Amplitude")
                .with_range(FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_default(0.8),
			oscillator1: ParameterGroup::new(GroupId(1), "Oscillator 1", OscillatorParams::new(10)),
			oscillator2: ParameterGroup::new(GroupId(2), "Oscillator 2", OscillatorParams::new(20))
        }
    }
}