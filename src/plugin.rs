use crate::AudioLayout;
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

pub trait Plugin {
    const NAME: &'static str;
    const VENDOR: &'static str;
    const URL: &'static str;
    const EMAIL: &'static str;
    const AUDIO_LAYOUT: &'static [AudioLayout];

    fn new() -> Self;
    fn reset(&mut self, sample_rate: f64);
    fn editor(&self) -> Option<Box<dyn Editor>> {
        None
    }

    //fn process(&mut self);
}