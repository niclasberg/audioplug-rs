mod envelope;

pub use envelope::{ADSREnvelope, ADSRParameters, AREnvelope, ARParameters};
use num::Float;

pub trait DspFloat: Float {
    fn from_f32(value: f32) -> Self;
}

impl DspFloat for f32 {
    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value
    }
}

impl DspFloat for f64 {
    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value as _
    }
}
