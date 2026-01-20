use std::time::Duration;

use uuid::Uuid;

use crate::editor::Editor;
use crate::midi::NoteEvent;
use crate::param::Params;
use crate::wrapper::vst3::VST3Categories;
use crate::{AudioBuffer, AudioLayout};

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str,
}

impl PluginInfo {
    pub fn for_plugin<P: Plugin>(_plugin: &P) -> Self {
        Self {
            name: P::NAME,
            vendor: P::VENDOR,
            url: P::URL,
            email: P::EMAIL,
        }
    }
}

pub struct Preset<P: Params> {
    pub name: String,
    pub parameters: P,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessInfo {
    pub rendering_offline: bool,
    pub sample_rate: f64,
}

pub struct ProcessContext<'a> {
    pub input: &'a AudioBuffer,
    pub output: &'a mut AudioBuffer,
    pub info: ProcessInfo,
}

pub struct MidiProcessContext {
    pub info: ProcessInfo,
}

pub trait Plugin: Send + 'static {
    /// Name of the plugin
    const NAME: &'static str;
    /// Name of the plugin vendor
    const VENDOR: &'static str;
    /// URL of the plugin vendor
    const URL: &'static str;
    const EMAIL: &'static str;
    const AUDIO_LAYOUT: AudioLayout;
    /// Type of editor (a.k.a. user interface) for the plugin.
    type Editor: Editor<Parameters = Self::Parameters>;
    type Parameters: Params;

    /// True if the plugin accepts midi input messages
    const ACCEPTS_MIDI: bool = false;
    /// True if the plugin produces output midi messages
    const PRODUCES_MIDI: bool = false;

    fn new() -> Self;

    /// Called before processing starts. If you need to allocate memory for internal buffers,
    /// this is where to do it. [`max_buffer_size`] is the maximal number of samples that
    /// the host can request to be processed in a single call to [process].
    fn prepare(&mut self, sample_rate: f64, max_buffer_size: usize);

    fn process(&mut self, context: ProcessContext, parameters: &Self::Parameters);

    fn process_midi(
        &mut self,
        _context: &mut MidiProcessContext,
        _parameters: &Self::Parameters,
        _event: NoteEvent,
    ) {
    }

    /// Called when the plugin should reset internal buffers and voices (???)
    fn reset(&mut self) {}

    fn presets(&self) -> Vec<Preset<Self::Parameters>> {
        Vec::new()
    }

    /// Length of the tail of the signal from the plugin. This can be thought of as the
    /// time it takes for the plugin to go silent when no more input is given. A good example
    /// for a plugin type with a tail is a reverb, which will reverberate for some time for
    /// each input value.
    fn tail_time(&self) -> Duration {
        Duration::ZERO
    }

    /// The latency (in number of samples) that the plugin imposes
    fn latency_samples(&self) -> usize {
        0
    }
}

pub trait VST3Plugin: Plugin {
    const PROCESSOR_UUID: Uuid;
    const EDITOR_UUID: Uuid;
    const CATEGORIES: VST3Categories;
}
