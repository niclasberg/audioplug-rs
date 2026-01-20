use std::ptr::NonNull;

use objc2::{msg_send, rc::Retained, AllocAnyThread};
use objc2_audio_toolbox::{
    kAudioUnitErr_NoConnection, AUAudioFrameCount, AUAudioUnitBus, AUAudioUnitBusArray,
    AUAudioUnitStatus, AURenderPullInputBlock, AudioUnitRenderActionFlags,
};
use objc2_avf_audio::AVAudioFormat;
use objc2_foundation::{NSError, NSInteger};

use objc2_core_audio_types::{self as core_audio, AudioTimeStamp};

use crate::AudioLayout;

pub struct BusBuffer {
    channel_count: usize,
    max_frames: usize,
    samples: Box<[f32]>,
    buffers: Box<[core_audio::AudioBuffer]>,
    buffer_list: core_audio::AudioBufferList,
    buses: Vec<Retained<AUAudioUnitBus>>,
}

fn create_audio_unit_bus(
    format: &AVAudioFormat,
) -> Result<Retained<AUAudioUnitBus>, Retained<NSError>> {
    unsafe { msg_send![AUAudioUnitBus::alloc(), initWithFormat: format, error:_] }
}

unsafe fn get_audio_bus_format(bus: &AUAudioUnitBus) -> &AVAudioFormat {
    msg_send![bus, format]
}

impl BusBuffer {
    pub fn new(channel_count: usize, format: &AVAudioFormat) -> Self {
        let bus = create_audio_unit_bus(format).unwrap();
        unsafe { bus.setMaximumChannelCount(channel_count as _) };
        let buses = vec![bus];

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
                mBuffers: [core_audio::AudioBuffer {
                    mNumberChannels: 0,
                    mDataByteSize: 0,
                    mData: std::ptr::null_mut(),
                }],
            },
            buses,
            buffers,
        }
    }

    pub fn buses(&self) -> &[Retained<AUAudioUnitBus>] {
        self.buses.as_ref()
    }

    pub fn allocate(&mut self, max_frames: usize) {}

    pub fn deallocate(&mut self) {}

    fn update_buffer_list(&mut self, frame_count: usize) {
        let samples_ptr = self.samples.as_mut_ptr();
    }

    pub fn pull_inputs(
        &mut self,
        action_flags: NonNull<AudioUnitRenderActionFlags>,
        timestamp: NonNull<AudioTimeStamp>,
        frame_count: AUAudioFrameCount,
        input_bus_number: NSInteger,
        pull_input_block: AURenderPullInputBlock,
    ) -> AUAudioUnitStatus {
        let Some(pull_input_block) = (unsafe { pull_input_block.as_ref() }) else {
            return kAudioUnitErr_NoConnection;
        };

        pull_input_block.call((
            action_flags,
            timestamp,
            frame_count,
            input_bus_number,
            NonNull::from(&mut self.buffer_list),
        ))
    }

    pub fn prepare_output_buffer_list(&self) {}

    pub fn sample_rate(&self) -> Option<f64> {
        self.buses
            .first()
            .map(|bus| unsafe { get_audio_bus_format(bus).sampleRate() })
    }
}

pub fn create_buffers(format: &AVAudioFormat, layout: &AudioLayout) -> (BusBuffer, BusBuffer) {
    let input_buffer = BusBuffer::new(2, &format);
    let output_buffer = BusBuffer::new(2, &format);
    (input_buffer, output_buffer)
}
