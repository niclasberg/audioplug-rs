use std::ffi::c_void;
use audioplug::core::{Color, Size};
use audioplug::param::{BoolParameter, FloatParameter, FloatRange, NormalizedValue, Parameter, ParameterId};
use audioplug::view::{AnyView, Column, Label, Slider, View};
use audioplug::window::AppContext;
use audioplug::{params, AudioLayout, Bus, ChannelType, Editor, Plugin, ProcessContext};
use audioplug::wrapper::vst3::Factory;

struct MyPlugin {

}

struct MyEditor {

}

impl Editor<MyPluginParams> for MyEditor {
	fn new() -> Self {
		Self {}
	}

	fn prefered_size(&self) -> Option<Size> {
		Some(Size::new(540.0, 480.0))
	}

    fn view(&self, _ctx: &mut AppContext, _parameters: &MyPluginParams) -> AnyView {
        Column::new((
            Label::new("Text input").with_color(Color::BLUE),
            Slider::new()
                .on_value_changed(|ctx, value| {
                    let id = ParameterId::new(2);
                    ctx.begin_edit(id);
                    ctx.perform_edit(id, NormalizedValue::from_f64(value).unwrap());
                    ctx.end_edit(id);
                })
        )).as_any()
    }
}

params!(
	struct MyPluginParams {
		enabled: BoolParameter,
		gain: FloatParameter
	}
);

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            enabled: BoolParameter::new(ParameterId::new(1), "Enabled", true),
            gain: FloatParameter::new(ParameterId::new(2), "Gain")
				.with_range(FloatRange::Linear { min: 0.0, max: 1.0 })
				.with_default(0.5)
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Gain Plugin";
    const VENDOR: &'static str = "Some vendor";
    const URL: &'static str = "www.example.com";
    const EMAIL: &'static str = "niclasbrg@gmail.com";
    const AUDIO_LAYOUT: &'static [AudioLayout] = &[
		AudioLayout {
			main_input: Some(Bus { name: "Stereo Input", channel: ChannelType::Stereo }),
			main_output: Some(Bus { name: "Stereo Output", channel: ChannelType::Stereo })
		}
	];
    type Editor = MyEditor;
    type Parameters = MyPluginParams;

    fn new() -> Self {
        Self {}
    }

    fn reset(&mut self, _sample_rate: f64) {
        
    }

    fn process(&mut self, ctx: ProcessContext, parameters: &MyPluginParams) {
        let gain = parameters.gain.value() as f32;
        for (in_channel, mut out_channel) in ctx.input.channels_iter().zip(ctx.output.channels_iter_mut()) {
            for (in_sample, out_sample) in in_channel.iter().zip(out_channel.iter_mut()) {
                *out_sample = in_sample * gain;
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


#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleEntry() -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn bundleExit() -> bool {
    true
}

#[cfg(target_os = "macos")]
use audioplug::wrapper::au::{ViewController, NSError};

#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn create_view_controller() -> *mut c_void {
	Box::into_raw(Box::new(ViewController::<MyPlugin>::new())) as *mut _
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn destroy_view_controller(view_controller: *mut c_void) {
    
	drop(unsafe { Box::from_raw(view_controller as *mut ViewController::<MyPlugin>) });
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn create_audio_unit(view_controller: *mut c_void, desc: audioplug::wrapper::au::audio_toolbox::AudioComponentDescription, error: *mut *mut NSError) -> *mut c_void {
	(&mut *(view_controller as *mut ViewController::<MyPlugin>)).create_audio_unit(desc, error) as *mut _
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn create_view(view_controller: *mut c_void) -> *mut c_void {
	(&mut *(view_controller as *mut ViewController::<MyPlugin>)).create_view() as *mut _
}