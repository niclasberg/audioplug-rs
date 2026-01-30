use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSTCategory {
    Fx,
    Instrument,
    Spatial,
    Analyzer,
    Bass,
    ChannelStrip,
    Delay,
    Distortion,
    EQ,
    Filter,
    Generator,
    Mastering,
    Modulation,
    PitchShift,
    Restoration,
    Reverb,
    Surround,
    Tools,
    Network,
    Drum,
    Sample,
    Synth,
    External,
    OnlyRT,
    OnlyOfflineProcess,
    NoOfflineProcess,
    UpDownMix,
}

impl VSTCategory {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Fx => "Fx",
            Self::Instrument => "Instrument",
            Self::Spatial => "Spatial",
            Self::Analyzer => "Analyzer",
            Self::Bass => "Bass",
            Self::ChannelStrip => "Channel Strip",
            Self::Delay => "Delay",
            Self::Distortion => "Distortion",
            Self::EQ => "EQ",
            Self::Filter => "Filter",
            Self::Generator => "Generator",
            Self::Mastering => "Mastering",
            Self::Modulation => "Modulation",
            Self::PitchShift => "Pitch Shift",
            Self::Restoration => "Restoration",
            Self::Reverb => "Reverb",
            Self::Surround => "Surround",
            Self::Tools => "Tools",
            Self::Network => "Network",
            Self::Drum => "Drum",
            Self::Sample => "Sampler",
            Self::Synth => "Synth",
            Self::External => "External",
            Self::OnlyRT => "OnlyRT",
            Self::OnlyOfflineProcess => "OnlyOfflineProcess",
            Self::NoOfflineProcess => "NoOfflineProcess",
            Self::UpDownMix => "Up-Downmix",
        }
    }
}

pub struct VST3Categories(&'static [VSTCategory]);

impl VST3Categories {
    pub const FX: Self = Self(&[VSTCategory::Fx]);
    /// Scope, FFT-Display, Loudness Processing...
    pub const FX_ANALYZER: Self = Self(&[VSTCategory::Fx, VSTCategory::Analyzer]);
    /// Tools dedicated to Bass Guitar.
    pub const FX_BASS: Self = Self(&[VSTCategory::Fx, VSTCategory::Bass]);
    /// Tools dedicated to Channel Strip.
    pub const FX_CHANNEL_STRIP: Self = Self(&[VSTCategory::Fx, VSTCategory::ChannelStrip]);
    /// Delay, Multi-tap Delay, Ping-Pong Delay...
    pub const FX_DELAY: Self = Self(&[VSTCategory::Fx, VSTCategory::Delay]);
    /// Amp Simulator, Sub-Harmonic, SoftClipper...
    pub const FX_DISTORTION: Self = Self(&[VSTCategory::Fx, VSTCategory::Distortion]);

    /// Volume, Mixer, Tuner...
    pub const FX_TOOLS: Self = Self(&[VSTCategory::Fx, VSTCategory::Tools]);

    /// Effect used as instrument (sound generator), not as insert.
    pub const INSTRUMENT: Self = Self(&[VSTCategory::Instrument]);
    /// Instrument for Drum sounds.
    pub const INSTRUMENT_DRUM: Self = Self(&[VSTCategory::Instrument, VSTCategory::Drum]);
    /// Instrument based on Synthesis.
    pub const INSTRUMENT_SYNTH: Self = Self(&[VSTCategory::Instrument, VSTCategory::Synth]);

    pub const fn new(categories: &'static [VSTCategory]) -> Self {
        Self(categories)
    }
}

impl Display for VST3Categories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .0
            .iter()
            .map(VSTCategory::as_str)
            .collect::<Vec<_>>()
            .join("|");
        f.write_str(&str)
    }
}
