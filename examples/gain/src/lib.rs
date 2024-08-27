use audioplug::core::{Color, Size};
use audioplug::param::{BoolParameter, FloatParameter, FloatRange, NormalizedValue, Parameter, ParameterId};
use audioplug::view::{AnyView, Column, Label, Slider, View};
use audioplug::window::AppContext;
use audioplug::{audioplug_auv3_plugin, audioplug_vst3_plugin, params, AudioLayout, Bus, ChannelType, Editor, Plugin, ProcessContext};

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

struct MyEditor;
impl Editor for MyEditor {
    type Parameters = MyPluginParams;

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

struct MyPlugin {

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

    fn reset(&mut self, _sample_rate: f64, _max_samples_per_frame: usize) {
        
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

audioplug_vst3_plugin!(MyPlugin);
audioplug_auv3_plugin!(MyPlugin);
