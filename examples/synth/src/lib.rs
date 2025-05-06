use core::f32;

use audioplug::{
    audioplug_auv3_plugin, audioplug_vst3_plugin,
    midi::{Note, NoteEvent},
    param::Parameter,
    wrapper::vst3::VST3Categories,
    AudioLayout, Bus, ChannelType, Plugin, VST3Plugin,
};
use editor::SynthEditor;
use params::SynthParams;
use voice::Voice;

mod editor;
mod params;
mod views;
mod voice;

struct SynthPlugin {
    active_voice: Option<Voice>,
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
            active_voice: None,
            dt: 0.0,
        }
    }

    fn prepare(&mut self, sample_rate: f64, _max_buffer_size: usize) {
        self.dt = 1.0 / sample_rate as f32;
    }

    fn process(&mut self, context: audioplug::ProcessContext, parameters: &Self::Parameters) {
        if let Some(voice) = &mut self.active_voice {
            for sample in context.output.channel_mut(0).iter_mut() {
                *sample = parameters.amplitude.value() as f32 * f32::sin(voice.ang_freq * voice.t);
                voice.t += self.dt;
            }
        } else {
            for sample in context.output.channel_mut(0).iter_mut() {
                *sample = 0.0;
            }
        }
    }

    fn process_midi(
        &mut self,
        context: &mut audioplug::MidiProcessContext,
        _parameters: &Self::Parameters,
        event: audioplug::midi::NoteEvent,
    ) {
        match event {
            NoteEvent::NoteOn { note, .. } => {
                self.active_voice.replace(Voice::new(
                    context.info.sample_rate as _,
                    Default::default(),
                ));
            }
            NoteEvent::NoteOff { note, .. } => {
                if self
                    .active_voice
                    .as_ref()
                    .is_some_and(|voice| voice.note == note)
                {
                    self.active_voice.take();
                }
            }
        }
    }

    fn reset(&mut self) {
        self.active_voice = None;
    }

    fn tail_time(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    fn latency_samples(&self) -> usize {
        0
    }
}

impl VST3Plugin for SynthPlugin {
    const PROCESSOR_UUID: [u8; 16] = *b"audioplugsynthpc";
    const EDITOR_UUID: [u8; 16] = *b"audioplugsynthed";
    const CATEGORIES: VST3Categories = VST3Categories::INSTRUMENT_SYNTH;
}

audioplug_vst3_plugin!(SynthPlugin);
audioplug_auv3_plugin!(SynthPlugin);
