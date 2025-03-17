use audioplug::{param::{BoolParameter, FloatParameter, FloatRange, GroupId, IntParameter, ParameterGroup, ParameterId, Params}, params};

params!(
	pub struct OscillatorParams {
		pub octave: IntParameter,
		pub semitones: IntParameter
	}
);

impl OscillatorParams {
	pub fn new(offset: u32) -> Self {
		Self {
			octave: IntParameter::new(ParameterId(offset + 0), "Octave")
				.with_range(-3..=3),
			semitones: IntParameter::new(ParameterId(offset + 1), "Semitones")
				.with_range(-11..=11)
		}
	}
}

params!(
	pub struct FilterParams {
		pub enabled: BoolParameter,
		pub cutoff: FloatParameter,
		pub resonance: FloatParameter
	}
);

impl FilterParams {
	pub fn new(offset: u32) -> Self {
		Self {
			enabled: BoolParameter::new(ParameterId(offset + 0), "Enabled", true),
			cutoff: FloatParameter::new(ParameterId(offset + 1), "Filter Cutoff")
				.with_linear_range(100.0, 3000.0),
			resonance: FloatParameter::new(ParameterId(offset + 2), "Filter Resonance")
				.with_linear_range(0.0, 1.0)
		}
	}
}

params!(
    pub struct SynthParams {
        pub amplitude: FloatParameter,
		pub filter: ParameterGroup<FilterParams>,
		pub oscillators: [ParameterGroup<OscillatorParams>; 2]
    }
);

impl Params for SynthParams {
    fn new() -> Self {
        Self {  
            amplitude: FloatParameter::new(ParameterId(1), "Amplitude")
                .with_range(FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_default(0.8),
			filter: ParameterGroup::new(GroupId(1), "Filter", FilterParams::new(10)),
			oscillators: [
				ParameterGroup::new(GroupId(2), "Oscillator 1", OscillatorParams::new(20)),
				ParameterGroup::new(GroupId(3), "Oscillator 2", OscillatorParams::new(30))
			]
        }
    }
}