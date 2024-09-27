use audioplug::{param::{FloatParameter, FloatRange, ParameterId}, params};

params!(
    pub struct SynthParams {
        pub amplitude: FloatParameter
    }
);

impl Default for SynthParams {
    fn default() -> Self {
        Self {  
            amplitude: FloatParameter::new(ParameterId::new(1), "Amplitude")
                .with_range(FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_default(0.8)
        }
    }
}