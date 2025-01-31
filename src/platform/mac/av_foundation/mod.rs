use block2::Block;
use objc2::{extern_class, extern_methods, rc::{Allocated, Retained}, AllocAnyThread};
use objc2_foundation::{NSError, NSObject, NSString};

use super::audio_toolbox::{AUAudioUnit, AudioComponentDescription, AudioComponentInstantiationOptions};

extern_class!(
	#[unsafe(super(NSObject))]
	#[derive(PartialEq, Eq, Hash)]
	pub struct AVAudioNode;
);

extern_class!(
	#[unsafe(super(AVAudioNode))]
	#[derive(PartialEq, Eq, Hash)]
	pub struct AVAudioUnit;
);

impl AVAudioUnit {
	extern_methods!(
		#[unsafe(method(instantiateWithComponentDescription:options:completionHandler:))]
		#[allow(non_snake_case)]
		pub fn instantiateWithComponentDescription_options_completionHandler(
			desc: AudioComponentDescription, 
			options: AudioComponentInstantiationOptions,
			completion_handler: &Block<dyn Fn(*mut AVAudioUnit, *mut NSError)>);

		#[unsafe(method(audioComponentDescription))]
		pub fn audio_component_description(&self) -> AudioComponentDescription;

		#[unsafe(method(manufacturerName))]
		pub fn manufacturer_name(&self) -> Retained<NSString>;

		#[unsafe(method(name))]
		pub fn name(&self) -> Retained<NSString>;

		#[unsafe(method(AUAudioUnit))]
		pub fn au_audio_unit(&self) -> Retained<AUAudioUnit>;
	);
}

pub type AVAudioChannelCount = u32;

extern_class!(
	#[unsafe(super(NSObject))]
	pub struct AVAudioFormat;
);

impl AVAudioFormat {
	extern_methods!(
		#[unsafe(method(initStandardFormatWithSampleRate:channels:))]
		#[allow(non_snake_case)]
		pub unsafe fn initStandardFormatWithSampleRate_channels(
			this: Allocated<Self>,
			sampleRate: f64,
			channels: AVAudioChannelCount) -> Retained<Self>;
	);
}

impl AVAudioFormat {
	pub fn new_with_sample_rate_and_channels(sample_rate: f64, channels: AVAudioChannelCount) -> Retained<Self> {
		unsafe {
			AVAudioFormat::initStandardFormatWithSampleRate_channels(AVAudioFormat::alloc(), sample_rate, channels)
		}
	}
}