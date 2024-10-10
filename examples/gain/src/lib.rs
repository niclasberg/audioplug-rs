use audioplug::core::{Color, Size};
use audioplug::param::{BoolParameter, FloatParameter, FloatRange, Parameter, ParameterId, Params};
use audioplug::view::{AnyView, Column, Label, ParameterSlider, View};
use audioplug::app::ViewContext;
use audioplug::{audioplug_auv3_plugin, audioplug_vst3_plugin, params, AudioLayout, Bus, ChannelType, Editor, Plugin, ProcessContext, VST3Plugin};

params!(
	struct MyPluginParams {
		enabled: BoolParameter,
		gain: FloatParameter
	}
);

impl Params for MyPluginParams {
    fn new() -> Self {
        Self {
            enabled: BoolParameter::new(ParameterId::new(1), "Enabled", true),
            gain: FloatParameter::new(ParameterId::new(2), "Gain")
				.with_range(FloatRange::Linear { min: 0.0, max: 1.0 })
				.with_default(0.5)
        }
    }
}

struct MyEditor;
impl Editor for MyEditor {
    type Parameters = MyPluginParams;

	fn new() -> Self {
		Self {}
	}

	fn prefered_size(&self) -> Option<Size> {
		Some(Size::new(540.0, 480.0))
	}

    fn view(&self, _ctx: &mut ViewContext, parameters: &MyPluginParams) -> AnyView {
        Column::new((
            Label::new("Gain").with_color(Color::BLUE),
            ParameterSlider::new(&parameters.gain)
        ))
		.padding(10.0)
		.as_any()
    }
}

struct MyPlugin {

}

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Gain Plugin";
    const VENDOR: &'static str = "Audioplug";
    const URL: &'static str = "https://github.com/niclasberg/audioplug-rs";
    const EMAIL: &'static str = "niclasbrg@gmail.com";
    const AUDIO_LAYOUT: AudioLayout = AudioLayout {
		main_input: Some(Bus { name: "Stereo Input", channel: ChannelType::Stereo }),
		main_output: Some(Bus { name: "Stereo Output", channel: ChannelType::Stereo })
	};
    type Editor = MyEditor;
    type Parameters = MyPluginParams;

    fn new() -> Self {
        Self {}
    }

    fn prepare(&mut self, _sample_rate: f64, _max_samples_per_frame: usize) {
        
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

impl VST3Plugin for MyPlugin {
	const PROCESSOR_UUID: [u8; 16] = *b"audiopluggainprc";
	const EDITOR_UUID: [u8; 16] = *b"audiopluggainedt";
}

audioplug_vst3_plugin!(MyPlugin);
audioplug_auv3_plugin!(MyPlugin);
