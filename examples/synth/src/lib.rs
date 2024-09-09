use audioplug::{audioplug_auv3_plugin, audioplug_vst3_plugin, AudioLayout, Bus, ChannelType, GenericEditor, Plugin};
use params::SynthParams;

mod views;
mod params;

struct SynthPlugin;
impl Plugin for SynthPlugin {
    const NAME: &'static str = "Synth";
    const VENDOR: &'static str = "Vendor";
    const URL: &'static str = "https://github.com/niclasberg";
    const EMAIL: &'static str = "some@email.com";
    const AUDIO_LAYOUT: &'static [AudioLayout] = &[
        AudioLayout {
            main_input: None,
            main_output: Some(Bus { name: "Stereo Output", channel: ChannelType::Stereo })
        }
    ];
    const ACCEPTS_MIDI: bool = true;

    type Editor = GenericEditor<SynthParams>;
    type Parameters = SynthParams;

    fn new() -> Self {
        Self
    }

    fn reset(&mut self, _sample_rate: f64, _max_buffer_size: usize) {
        
    }

    fn process(&mut self, _context: audioplug::ProcessContext, _parameters: &Self::Parameters) {
        
    }
    
    fn tail_time(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
    
    fn latency_samples(&self) -> usize {
        0
    }
}

audioplug_vst3_plugin!(SynthPlugin);
audioplug_auv3_plugin!(SynthPlugin);
