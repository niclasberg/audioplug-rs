use audioplug::wrapper::au::AudioUnit;
use audioplug::wrapper::au::audio_toolbox::{AUAudioUnit, AudioComponentDescription};
use objc2::rc::autoreleasepool;
use objc2::ClassType;
use objc2_foundation::{ns_string, NSError};

const fn four_cc(str: &[u8; 4]) -> u32 {
	((str[0] as u32) << 24 & 0xff000000)
	| ((str[1] as u32) << 16 & 0x00ff0000)
	| ((str[2] as u32) << 8 & 0x0000ff00)
	| ((str[3] as u32) & 0x000000ff)
}

fn main() {
	let desc: AudioComponentDescription = AudioComponentDescription {
		component_type: four_cc(b"aufx"),
		component_sub_type: four_cc(b"demo"),
		manufacturer: four_cc(b"nibe"),
		componentFlags: 0,
		componentFlagsMask: 0,
	};
	AUAudioUnit::registerSubclass(AudioUnit::class(), desc, ns_string!("RUST: TEST"), 0);
	
	let audio_unit = unsafe {
		AudioUnit::new_with_component_descriptor_error(desc, std::ptr::null_mut())
	};

	let buses = audio_unit.inputBusses();

	/*if let Err(e) = audio_unit {
		println!("{}", e);
		let code = e.code();
		autoreleasepool(|pool| {
			let domain = e.domain().as_str(pool);
			let desc = e.localizedDescription().as_str(pool);
			let b = "ASDF";
		});
	}*/
	
	println!("asdfas");
}