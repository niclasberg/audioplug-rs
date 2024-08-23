use block2::Block;
use objc2::{extern_class, extern_methods, mutability, rc::{Allocated, Retained}, ClassType};
use objc2_foundation::{NSError, NSObject, NSString};

use super::audio_toolbox::{AUAudioUnit, AudioComponentDescription, AudioComponentInstantiationOptions};

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AVAudioNode;

	unsafe impl ClassType for AVAudioNode {
		type Super = NSObject;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AVAudioUnit;

	unsafe impl ClassType for AVAudioUnit {
		type Super = AVAudioNode;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AVAudioUnit {
		#[method(instantiateWithComponentDescription:options:completionHandler:)]
		#[allow(non_snake_case)]
		pub fn instantiateWithComponentDescription_options_completionHandler(
			desc: AudioComponentDescription, 
			options: AudioComponentInstantiationOptions,
			completion_handler: &Block<dyn Fn(*mut AVAudioUnit, *mut NSError)>);

		#[method(audioComponentDescription)]
		pub fn audio_component_description(&self) -> AudioComponentDescription;

		#[method_id(manufacturerName)]
		pub fn manufacturer_name(&self) -> Retained<NSString>;

		#[method_id(name)]
		pub fn name(&self) -> Retained<NSString>;

		#[method_id(AUAudioUnit)]
		pub fn au_audio_unit(&self) -> Retained<AUAudioUnit>;
	}
);

pub type AVAudioChannelCount = u32;

extern_class!(
	pub struct AVAudioFormat;

	unsafe impl ClassType for AVAudioFormat {
		type Super = NSObject;
		type Mutability = objc2::mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AVAudioFormat {
		#[method_id(initStandardFormatWithSampleRate:channels:)]
		#[allow(non_snake_case)]
		pub unsafe fn initStandardFormatWithSampleRate_channels(
			this: Allocated<Self>,
			sampleRate: f64,
			channels: AVAudioChannelCount) -> Retained<Self>;
	}
);

impl AVAudioFormat {
	pub fn new_with_sample_rate_and_channels(sample_rate: f64, channels: AVAudioChannelCount) -> Retained<Self> {
		unsafe {
			AVAudioFormat::initStandardFormatWithSampleRate_channels(AVAudioFormat::alloc(), sample_rate, channels)
		}
	}
}