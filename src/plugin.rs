use crate::param::Params;
use crate::{AudioLayout, AudioBuffer};
use crate::editor::Editor;

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str
}

impl PluginInfo {
    pub fn for_plugin<P: Plugin>(_plugin: &P) -> Self {
        Self {
            name: P::NAME,
            vendor: P::VENDOR,
            url: P::URL,
            email: P::EMAIL
        }
    }
}

pub struct ProcessContext<'a> {
    pub input: &'a AudioBuffer,
    pub output: &'a mut AudioBuffer
}

pub trait Plugin {
    const NAME: &'static str;
    const VENDOR: &'static str;
    const URL: &'static str;
    const EMAIL: &'static str;
    const AUDIO_LAYOUT: &'static [AudioLayout];
    type Editor: Editor;
    type Parameters: Params;

    fn new() -> Self;
    fn reset(&mut self, sample_rate: f64);
    fn editor(&self) -> Self::Editor;

    fn process(&mut self, context: ProcessContext, parameters: &Self::Parameters);
}