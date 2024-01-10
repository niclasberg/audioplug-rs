use std::ffi::c_void;
use audioplug::core::Color;
use audioplug::views::Label;
use audioplug::{Plugin, AudioLayout, Bus, ChannelType, ProcessContext, Editor};
use audioplug::vst3::Factory;

struct MyPlugin {

}

struct MyEditor {

}

impl Editor for MyEditor {
    fn view(&self) -> impl audioplug::View {
        Label::new("Text input").with_color(Color::BLUE)
    }
}

struct OscillatorParams {
    enabled: bool,
    detune: f64,
    pos_x: f64,
    pos_y: f64,
    // waveform choice
    amplitude: f64
}

struct MyPluginParams {
    oscillators: [OscillatorParams; 4],   
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "Some vendor";
    const URL: &'static str = "www.example.com";
    const EMAIL: &'static str = "niclasbrg@gmail.com";
    const AUDIO_LAYOUT: &'static [AudioLayout] = &[AudioLayout {
        main_input: Some(Bus { name: "Stereo Input", channel: ChannelType::Stereo }),
        main_output: Some(Bus { name: "Stereo Output", channel: ChannelType::Stereo })
    }];
    type Editor = MyEditor;

    fn new() -> Self {
        Self {}
    }

    fn reset(&mut self, _sample_rate: f64) {
        
    }

    fn editor(&self) -> Self::Editor {
        MyEditor {}
    }

    fn process(&mut self, ctx: ProcessContext) {
        for (in_channel, mut out_channel) in ctx.input.channels_iter().zip(ctx.output.channels_iter_mut()) {
            for (in_sample, out_sample) in in_channel.iter().zip(out_channel.iter_mut()) {
                *out_sample = in_sample * 0.5;
            }
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
    Box::into_raw(Factory::<MyPlugin>::new()) as *mut c_void
}

#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn InitDll() -> bool {
    true
}

#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn ExitDll() -> bool {
    true
}