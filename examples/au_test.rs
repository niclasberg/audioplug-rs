#[cfg(target_os = "macos")] 
static IS_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(target_os = "macos")]
fn main() {
    use audioplug::wrapper::au::audio_toolbox::{AudioComponentFlags, AudioComponentInstantiationOptions};
	use audioplug::wrapper::au::av_foundation::AVAudioUnit;
	use audioplug::wrapper::au::AudioUnit;
	use audioplug::wrapper::au::audio_toolbox::{AUAudioUnit, AudioComponentDescription};
	use block2::StackBlock;
	use objc2::rc::autoreleasepool;
	use objc2::ClassType;
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
		flags_mask: 0,
	};
	//AUAudioUnit::registerSubclass(AudioUnit::class(), desc, ns_string!("RUST: TEST"), 0);
	
	let audio_unit = unsafe {
		AudioUnit::new_with_component_descriptor_error(desc, std::ptr::null_mut())
	};

	let block = StackBlock::new(move |unit, error| {
		IS_DONE.store(true, std::sync::atomic::Ordering::Release);
	});
	AVAudioUnit::instantiateWithComponentDescription_options_completionHandler(
		desc, 
		AudioComponentInstantiationOptions::kAudioComponentInstantiation_LoadOutOfProcess, 
		&block);

	while !IS_DONE.load(std::sync::atomic::Ordering::Acquire) {

	}

	let buses = audio_unit.inputBusses();
	
	println!("asdfas");
}


#[cfg(not(target_os = "macos"))]
fn main() {}