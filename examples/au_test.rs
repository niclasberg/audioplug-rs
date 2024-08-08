use std::sync::atomic::AtomicBool;

use audioplug::{GenericEditor, Plugin};

static IS_DONE: AtomicBool = AtomicBool::new(false);

struct TestPlugin {

}

impl Plugin for TestPlugin {
    const NAME: &'static str = "test";
    const VENDOR: &'static str = "test";
    const URL: &'static str = "www.test.com";
    const EMAIL: &'static str = "test@test.com";
    const AUDIO_LAYOUT: &'static [audioplug::AudioLayout] = &[];
    type Editor = GenericEditor<()>;
    type Parameters = ();

    fn new() -> Self {
        Self {}
    }

    fn reset(&mut self, sample_rate: f64) {
        
    }

    fn editor(&self) -> Self::Editor {
        GenericEditor::new()
    }

    fn process(&mut self, context: audioplug::ProcessContext, _parameters: &()) {
        
    }
    
}

#[cfg(target_os = "macos")]
fn main() {
    use audioplug::wrapper::au::audio_toolbox::{AUAudioUnitBusArray, AudioComponentFlags, AudioComponentInstantiationOptions};
	use audioplug::wrapper::au::av_foundation::AVAudioUnit;
	use audioplug::wrapper::au::audio_toolbox::{AUAudioUnit, AudioComponentDescription};
	use audioplug::wrapper::au::{MyAudioUnit, ViewController};
	use block2::StackBlock;
	use objc2::rc::autoreleasepool;
	use objc2::{msg_send, ClassType};
	use objc2_foundation::{ns_string, NSError};

	const fn four_cc(str: &[u8; 4]) -> u32 {
		((str[0] as u32) << 24 & 0xff000000)
		| ((str[1] as u32) << 16 & 0x00ff0000)
		| ((str[2] as u32) << 8 & 0x0000ff00)
		| ((str[3] as u32) & 0x000000ff)
	}

	let desc: AudioComponentDescription = AudioComponentDescription {
		component_type: four_cc(b"aufx"),
		component_sub_type: four_cc(b"demo"),
		manufacturer: four_cc(b"Nibe"),
		flags: AudioComponentFlags(0),
		flags_mask: 0x12,
	};

	// let view_controller: ViewController<TestPlugin> = ViewController::new();
	// view_controller.create_audio_unit(desc, error)
	
	let audio_unit = MyAudioUnit::<TestPlugin>::try_with_component_descriptor(desc).unwrap();	
	let input_busses = audio_unit.input_busses();
	let output_busses = audio_unit.output_busses();

	let aa = input_busses.count();
	let bb = unsafe { audio_unit.allocate_render_resources() };

	let render_block = audio_unit.render_block();

	unsafe {
		let render_block = &*render_block;
	}

	//AUAudioUnit::registerSubclass(AudioUnit::class(), desc, ns_string!("RUST: TEST"), 0);

	let block = StackBlock::new(move |unit, error| {
		let audio_unit: Option<&AUAudioUnit> = unsafe { msg_send![unit, AUAudioUnit] };

		IS_DONE.store(true, std::sync::atomic::Ordering::Release);
	});
	AVAudioUnit::instantiateWithComponentDescription_options_completionHandler(
		desc, 
		AudioComponentInstantiationOptions::kAudioComponentInstantiation_LoadOutOfProcess, 
		&block);

	while !IS_DONE.load(std::sync::atomic::Ordering::Acquire) {

	}
	
	println!("asdfas");
}


#[cfg(not(target_os = "macos"))]
fn main() {}