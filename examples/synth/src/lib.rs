use core::f32;

use audioplug::{
    audioplug_auv3_plugin, audioplug_clap_plugin, audioplug_vst3_plugin,
    midi::NoteEvent,
    wrapper::{clap::ClapFeature, vst3::VST3Categories},
    AudioLayout, Bus, ChannelType, ClapPlugin, Plugin, Uuid, VST3Plugin,
};
use editor::SynthEditor;
use params::SynthParams;
use voice::Voice;

mod editor;
mod params;
mod views;
mod voice;

struct SynthPlugin {
    active_voice: Voice,
    dt: f32,
}

impl Plugin for SynthPlugin {
    const NAME: &'static str = "Synth";
    const VENDOR: &'static str = "Audioplug";
    const URL: &'static str = "https://github.com/niclasberg/audioplug-rs";
    const EMAIL: &'static str = "some@email.com";
    const AUDIO_LAYOUT: AudioLayout = AudioLayout {
        main_input: None,
        main_output: Some(Bus {
            name: "Stereo Output",
            channel: ChannelType::Stereo,
        }),
    };
    const ACCEPTS_MIDI: bool = true;

    type Editor = SynthEditor;
    type Parameters = SynthParams;

    fn new() -> Self {
        Self {
            active_voice: Voice::new(48000.0, Default::default()),
            dt: 0.0,
        }
    }

    fn prepare(&mut self, sample_rate: f64, _max_buffer_size: usize) {
        self.dt = 1.0 / sample_rate as f32;
    }

    fn process(&mut self, context: audioplug::ProcessContext, parameters: &Self::Parameters) {
        for sample in context.output.channel_mut(0).iter_mut() {
            *sample = parameters.amplitude.value() as f32
                * f32::sin(self.active_voice.ang_freq * self.active_voice.t);
            self.active_voice.t += self.dt;
        }
    }

    fn process_midi(
        &mut self,
        _context: &mut audioplug::MidiProcessContext,
        _parameters: &Self::Parameters,
        event: audioplug::midi::NoteEvent,
    ) {
        match event {
            NoteEvent::NoteOn { note, .. } => {
                self.active_voice.note_on(note);
            }
            NoteEvent::NoteOff { note, .. } => {
                if self.active_voice.note == note {
                    self.active_voice.note_off();
                }
            }
        }
    }

    fn reset(&mut self) {
        self.active_voice.reset();
    }

    fn tail_time(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    fn latency_samples(&self) -> usize {
        0
    }
}

impl VST3Plugin for SynthPlugin {
    const PROCESSOR_UUID: Uuid = Uuid::from_bytes(*b"audioplugsynthpc");
    const EDITOR_UUID: Uuid = Uuid::from_bytes(*b"audioplugsynthed");
    const CATEGORIES: VST3Categories = VST3Categories::INSTRUMENT_SYNTH;
}

impl ClapPlugin for SynthPlugin {
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

audioplug_vst3_plugin!(SynthPlugin);
audioplug_auv3_plugin!(SynthPlugin);
audioplug_clap_plugin!(SynthPlugin);
