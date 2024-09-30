use audioplug::{param::{FloatParameter, FloatRange, ParameterId, Params}, params};

params!(
    pub struct SynthParams {
        pub amplitude: FloatParameter
    }
);

impl Params for SynthParams {
    fn new() -> Self {
        Self {  
            amplitude: FloatParameter::new(ParameterId::new(1), "Amplitude")
                .with_range(FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_default(0.8)
        }
    }
}