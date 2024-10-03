use objc2::{rc::Retained, ClassType};
use objc2_foundation::NSInteger;

use crate::platform::{audio_toolbox::{AUAudioFrameCount, AUAudioUnitBus, AUAudioUnitBusArray, AUAudioUnitStatus, AURenderPullInputBlock, AudioUnitRenderActionFlags}, av_foundation::AVAudioFormat, core_audio};

pub struct BusBuffers {
	au_buffers: Retained<AUAudioUnitBusArray>,
}

pub struct BusBuffer {
	channel_count: usize,
	max_frames: usize,
	samples: Box<[f32]>,
	buffers: Box<[core_audio::AudioBuffer]>,
	buffer_list: core_audio::AudioBufferList,
	bus: Retained<AUAudioUnitBus>,
}

impl BusBuffer {
	pub fn new(channel_count: usize, format: &AVAudioFormat) -> Self {
		let bus = unsafe { 
			AUAudioUnitBus::initWithFormat_error(
				AUAudioUnitBus::alloc(),
				&format).unwrap()
		};
		bus.setMaximumChannelCount(channel_count as _);

		let mut buffers = Vec::new();
		for _ in 0..channel_count {
			buffers.push(core_audio::AudioBuffer {
				mNumberChannels: channel_count as _,
				mDataByteSize: 0,
				mData: std::ptr::null_mut(),
			})
		}
		let buffers = buffers.into_boxed_slice();

		Self {
			channel_count,
			max_frames: 0,
			samples: Box::new([]),
			buffer_list: core_audio::AudioBufferList {
				mNumberBuffers: 0,
				mBuffers: std::ptr::null_mut(),
			},
			bus,
			buffers
		}
	}

	pub fn bus(&self) -> &AUAudioUnitBus {
		&self.bus
	}

	pub fn allocate(&mut self, max_frames: usize) {
		
	}

	pub fn deallocate(&mut self) {
		
	}

	fn update_buffer_list(&mut self, frame_count: usize) {
		let samples_ptr = self.samples.as_mut_ptr();

	}

	pub fn pull_inputs(&mut self, 
		action_flags: *mut AudioUnitRenderActionFlags, 
		timestamp: *const core_audio::AudioTimeStamp, 
		frame_count: AUAudioFrameCount,
		input_bus_number: NSInteger,
		pull_input_block: Option<&AURenderPullInputBlock>
	) -> AUAudioUnitStatus {
		let Some(pull_input_block) = pull_input_block else { return -10876 }; // NoConnection 

		pull_input_block.call((action_flags, timestamp, frame_count, input_bus_number, &mut self.buffer_list as *mut core_audio::AudioBufferList))
	}

	pub fn prepare_output_buffer_list(&self) {

	}
}