use std::cell::Cell;
use std::ptr::NonNull;
use std::{cell::OnceCell, rc::Rc, sync::Arc};

use atomic_refcell::AtomicRefCell;
use block2::{Block, RcBlock};
use objc2::{define_class, extern_class, extern_methods, msg_send, rc::Retained, runtime::Bool, AllocAnyThread, DeclaredClass};
use objc2_audio_toolbox::{AUAudioFrameCount, AUAudioUnit, AUAudioUnitBusArray, AUAudioUnitBusType, AUAudioUnitStatus, AUParameterTree, AURenderEventType, AURenderPullInputBlock, AudioComponentDescription, AudioUnitRenderActionFlags};
use objc2_avf_audio::AVAudioFormat;
use objc2_core_audio_types::{AudioBufferList, AudioTimeStamp};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSArray, NSError, NSIndexSet, NSInteger, NSNumber, NSObject, NSTimeInterval};

use crate::param::{AnyParameterMap, ParameterId, ParameterMap, Params, PlainValue};
use crate::{AudioBuffer, Plugin, ProcessContext};
use super::buffers::create_buffers;
use super::{buffers::BusBuffer, render_event::AURenderEvent, utils::create_parameter_tree};

const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

// Missing definitions from objc2
pub type AUInternalRenderBlock = Block<dyn Fn(NonNull<AudioUnitRenderActionFlags>, NonNull<AudioTimeStamp>, AUAudioFrameCount, NSInteger, *mut AudioBufferList, *const AURenderEvent, AURenderPullInputBlock) -> AUAudioUnitStatus>;
pub type AUInternalRenderRcBlock = RcBlock<dyn Fn(NonNull<AudioUnitRenderActionFlags>, NonNull<AudioTimeStamp>, AUAudioFrameCount, NSInteger, *mut AudioBufferList, *const AURenderEvent, AURenderPullInputBlock) -> AUAudioUnitStatus>;

extern_class!(
	#[unsafe(super(NSObject))]
	pub struct AUAudioUnitViewConfiguration;
);

impl AUAudioUnitViewConfiguration {
	extern_methods!(
		#[unsafe(method(width))]
		pub fn width(&self) -> CGFloat;

		#[unsafe(method(height))]
		pub fn height(&self) -> CGFloat;
	);
}

trait AnyWrapper {
	fn allocate_render_resources(&mut self, max_frames_to_render: usize);
	fn deallocate_render_resources(&mut self);
	fn latency(&self) -> NSTimeInterval;
	fn tail_time(&self) -> NSTimeInterval;
	fn render(&mut self, 
		action_flags: NonNull<AudioUnitRenderActionFlags>, 
		timestamp: NonNull<AudioTimeStamp>, 
		frame_count: AUAudioFrameCount, 
		output_bus_number: NSInteger, 
		output_data: *mut AudioBufferList, 
		realtime_event_list_head: *const AURenderEvent, 
		pull_input_block: AURenderPullInputBlock) -> AUAudioUnitStatus;
}

struct Wrapper<P: Plugin> {
	plugin: P,
    parameters: Rc<ParameterMap<P::Parameters>>,
	input_buffer: BusBuffer,
	output_buffer: BusBuffer,
	rendering_offline: bool
}

impl<P: Plugin> Wrapper<P> {
	fn process_events(&mut self, realtime_event_list_head: *const AURenderEvent) {
		let mut event_list = realtime_event_list_head;
		while !event_list.is_null() {
			let header = unsafe { &(&*event_list).head };
			//let ev_timestamp = header.event_sample_time;
			match header.event_type {
				AURenderEventType::Parameter | AURenderEventType::ParameterRamp => {
					let parameter_event = unsafe { &(&*event_list).parameter };
					let param_id = ParameterId(parameter_event.parameter_address as _);
					if let Some(param_ref) = self.parameters.get_by_id(param_id) {
						param_ref.set_value_plain(PlainValue::new(parameter_event.value as _));
					}
				},
				AURenderEventType::MIDI => {
					//let midi_event = unsafe { &(&*event_list).midi };
					
				},
				AURenderEventType::MIDIEventList => {
					//let midi_event_list = unsafe { &(&*event_list).midi_events_list };
				},
				_ => {}
			}
			event_list = header.next;
		}
	}
}

impl<P: Plugin> AnyWrapper for Wrapper<P> {
	fn render(&mut self, 
		_action_flags: NonNull<AudioUnitRenderActionFlags>, 
		_timestamp: NonNull<AudioTimeStamp>, 
		_frame_count: AUAudioFrameCount, 
		_output_bus_number: NSInteger, 
		_output_data: *mut AudioBufferList, 
		realtime_event_list_head: *const AURenderEvent, 
		_pull_input_block: AURenderPullInputBlock
	) -> AUAudioUnitStatus {
		self.process_events(realtime_event_list_head);

		let input = AudioBuffer::empty();
        let mut output = AudioBuffer::empty();

        let context = ProcessContext {
            input: &input,
            output: &mut output,
			rendering_offline: self.rendering_offline
        };

        self.plugin.process(context, self.parameters.parameters_ref());	
		0
	}

	fn allocate_render_resources(&mut self, max_frames_to_render: usize) {
		self.input_buffer.allocate(max_frames_to_render);
		self.output_buffer.allocate(max_frames_to_render);
	}

	fn deallocate_render_resources(&mut self) {
		self.input_buffer.deallocate();
		self.output_buffer.deallocate();
	}

	fn latency(&self) -> NSTimeInterval {
		self.plugin.latency_samples() as f64 / DEFAULT_SAMPLE_RATE
	}

	fn tail_time(&self) -> NSTimeInterval {
		self.plugin.tail_time().as_secs_f64()
	}
}

pub struct IVars {
	wrapper: Arc<AtomicRefCell<dyn AnyWrapper>>,
	internal_render_block: AUInternalRenderRcBlock,
	inputs: OnceCell<Retained<AUAudioUnitBusArray>>,
	outputs: OnceCell<Retained<AUAudioUnitBusArray>>,
	channel_capabilities: Retained<NSArray<NSNumber>>,
	parameter_tree: Retained<AUParameterTree>,
}

const CLASS_NAME: &'static str = match option_env!("AUDIOPLUG_AUDIO_UNIT_CLASS_NAME") {
	Some(name) => name,
	None => "AudioPlug_AudioUnit",
};

define_class!(
	#[unsafe(super(AUAudioUnit))]
	#[ivars = IVars]
	#[name = CLASS_NAME]
	pub struct MyAudioUnit;

	impl MyAudioUnit {
		#[unsafe(method_id(inputBusses))]
		fn input_busses(&self) -> Retained<AUAudioUnitBusArray> {
			self.ivars().inputs.get().unwrap().clone()
		}

		#[unsafe(method_id(outputBusses))]
		fn output_busses(&self) -> Retained<AUAudioUnitBusArray> {
			self.ivars().outputs.get().unwrap().clone()
		}

		#[unsafe(method_id(channelCapabilities))]
		fn __channel_capabilities(&self) -> Option<Retained<NSArray<NSNumber>>> {
			Some(self.ivars().channel_capabilities.clone())
		}

		#[unsafe(method(providesUserInterface))]
		fn provides_user_interface(&self) -> Bool {
			Bool::YES
		}

		#[unsafe(method_id(parameterTree))]
		fn __parameter_tree(&self) -> Option<Retained<AUParameterTree>> {
			Some(self.ivars().parameter_tree.clone())
		}

		#[unsafe(method_id(supportedViewConfiguations:))]
		fn supported_view_configurations(&self, available_view_configurations: &NSArray<AUAudioUnitViewConfiguration>) -> Option<Retained<NSIndexSet>> {
			Some(unsafe { NSIndexSet::indexSetWithIndexesInRange((0..available_view_configurations.count()).into()) }) 
		}

		#[unsafe(method(internalRenderBlock))]
		fn internal_render_block(&self) -> *mut AUInternalRenderBlock {
			RcBlock::into_raw(self.ivars().internal_render_block.clone())	
		}

		#[allow(non_snake_case)]
		#[unsafe(method(allocateRenderResourcesAndReturnError:))]
		fn allocateRenderResourcesAndReturnError(&self, error: *mut *mut NSError) -> Bool {
			let max_frames = unsafe { self.maximumFramesToRender() };
			self.ivars().wrapper.borrow_mut().allocate_render_resources(max_frames as _);
			unsafe { msg_send![super(self), allocateRenderResourcesAndReturnError: error] }
		}

		#[allow(non_snake_case)]
		#[unsafe(method(deallocateRenderResources))]
		fn deallocateRenderResources(&self) {
			self.ivars().wrapper.borrow_mut().deallocate_render_resources();
			unsafe { msg_send![super(self), deallocateRenderResources] }
		}

		#[unsafe(method(latency))]
		fn latency(&self) -> NSTimeInterval {
			self.ivars().wrapper.borrow().latency()
		}

		#[unsafe(method(tailTime))]
		fn tail_time(&self) -> NSTimeInterval {
			self.ivars().wrapper.borrow_mut().tail_time()
		}
	}
);

impl MyAudioUnit {
	pub fn new_with_component_descriptor_error<P: Plugin + 'static>(plugin: P, desc: AudioComponentDescription, out_error: *mut *mut NSError) -> Option<Retained<Self>> {
		let format = unsafe {
			AVAudioFormat::initStandardFormatWithSampleRate_channels(AVAudioFormat::alloc(), DEFAULT_SAMPLE_RATE, 2)
		}.unwrap();

		let parameters = ParameterMap::new(P::Parameters::new());
		let parameter_tree = create_parameter_tree(parameters.clone());
		let (input_buffer, output_buffer) = create_buffers(&format, &P::AUDIO_LAYOUT);

		let input_bus_array = NSArray::from_retained_slice(input_buffer.buses());
		let output_bus_array = NSArray::from_retained_slice(output_buffer.buses());

		let wrapper = Wrapper {
			plugin,
			parameters,
			input_buffer,
			output_buffer,
			rendering_offline: false,
		};

		let channel_capabilities = NSArray::from_retained_slice(&[
			NSNumber::new_i16(2), 
			NSNumber::new_i16(2)]
		);
		let wrapper = Arc::new(AtomicRefCell::new(wrapper));
		let internal_render_block = {
			let wrapper = wrapper.clone();
			AUInternalRenderRcBlock::new(move |flags, timestamp, frame_count, channels, buffers, events, pull_input_block| -> AUAudioUnitStatus {
				wrapper.borrow_mut().render(flags, timestamp, frame_count, channels, buffers, events, pull_input_block)
			})
		};

		let this = Self::alloc().set_ivars(IVars {
			wrapper,
			internal_render_block,
			inputs: OnceCell::new(),
			outputs: OnceCell::new(),
			channel_capabilities,
			parameter_tree,
		});
		
		let this: Option<Retained<Self>> = unsafe {
			msg_send!(super(this), initWithComponentDescription: desc, error: out_error)
		};

		let Some(this) = this else { return None };

		this.ivars().inputs.set(unsafe {
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				&this, 
				AUAudioUnitBusType::Input,
				&input_bus_array)
		}).unwrap();
		this.ivars().outputs.set(unsafe {
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				&this, 
				AUAudioUnitBusType::Output,
				&output_bus_array)
		}).unwrap();

		Some(this)
	}
}
