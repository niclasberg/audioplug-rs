use std::{cell::RefCell, ffi::{CStr, CString}, marker::PhantomData, mem::MaybeUninit, ops::Deref, rc::Rc, sync::OnceLock};

use atomic_refcell::AtomicRefCell;
use block2::RcBlock;
use objc2::{define_class, msg_send, rc::{Allocated, Retained}, runtime::{AnyClass, Bool, ClassBuilder, Sel}, sel, AllocAnyThread, ClassType, DefinedClass, Encoding, Message, RefEncode};
use objc2_core_audio_types::{AudioBufferList, AudioTimeStamp};
use objc2_foundation::{NSArray, NSError, NSIndexSet, NSInteger, NSNumber, NSObject, NSTimeInterval};
use crate::{param::{AnyParameterMap, ParameterId, ParameterMap, Params, PlainValue}, platform::{audio_toolbox::{AUAudioFrameCount, AUAudioUnit, AUAudioUnitBusArray, AUAudioUnitBusType, AUAudioUnitStatus, AUAudioUnitViewConfiguration, AUInternalRenderBlock, AUInternalRenderRcBlock, AUParameter, AUParameterTree, AURenderEvent, AURenderEventType, AURenderPullInputBlock, AUValue, AudioComponentInstantiationOptions, AudioUnitRenderActionFlags}, av_foundation::AVAudioFormat}, AudioBuffer, Plugin, ProcessContext};

use super::{buffers::BusBuffer, AudioComponentDescription};

const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

struct Wrapper<P: Plugin> {
	plugin: P,
    parameters: Rc<ParameterMap<P::Parameters>>,
	input_buffer: BusBuffer,
	output_buffer: BusBuffer,
	channel_capabilities: Retained<NSArray<NSNumber>>,
	internal_render_block: Option<AUInternalRenderRcBlock>,
	max_frames_to_render: usize,
	rendering_offline: bool
}

impl<P: Plugin + 'static> Wrapper<P> {
	pub fn new(audio_unit: &AUAudioUnit) -> Self {
		let plugin = P::new();
		

		let format = unsafe {
			AVAudioFormat::initStandardFormatWithSampleRate_channels(
				AVAudioFormat::alloc(), 
				DEFAULT_SAMPLE_RATE, 
				2)
		};

		let input_buffer = BusBuffer::new(2, &format);
		let inputs = unsafe {
			let bus_array = NSArray::from_slice(&[input_buffer.bus()]);
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				audio_unit, 
				AUAudioUnitBusType::Input,
				&bus_array)
		};

		let output_buffer = BusBuffer::new(2, &format);
		let outputs = unsafe {
			let bus_array = NSArray::from_slice(&[output_buffer.bus()]);
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				audio_unit, 
				AUAudioUnitBusType::Output,
				&bus_array)
		};

		let channel_capabilities = NSArray::from_retained_slice(&[
			NSNumber::new_i16(2), 
			NSNumber::new_i16(2)]
		);
		
		Self {
			plugin,
			parameters,
			input_buffer,
			output_buffer,
			channel_capabilities,
			internal_render_block: None,
			max_frames_to_render: 1024,
			rendering_offline: false
		}
	}

	pub fn render(&mut self, 
		_action_flags: *mut AudioUnitRenderActionFlags, 
		_timestamp: *const AudioTimeStamp, 
		_frame_count: AUAudioFrameCount, 
		_output_bus_number: NSInteger, 
		_output_data: *mut AudioBufferList, 
		realtime_event_list_head: *const AURenderEvent, 
		_pull_input_block: Option<&AURenderPullInputBlock>
	) {
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

		let input = AudioBuffer::empty();
        let mut output = AudioBuffer::empty();

        let context = ProcessContext {
            input: &input,
            output: &mut output,
			rendering_offline: self.rendering_offline
        };

        self.plugin.process(context, self.parameters.parameters_ref());	
	}

	pub fn allocate_render_resources(&mut self) {
		self.input_buffer.allocate(self.max_frames_to_render);
		self.output_buffer.allocate(self.max_frames_to_render);
	}

	pub fn deallocate_render_resources(&mut self) {
		self.input_buffer.deallocate();
		self.output_buffer.deallocate();
	}

	pub unsafe fn create_render_block(this: *mut Self) {
		let block = AUInternalRenderRcBlock::new(move |flags, timestamp, frame_count, channels, buffers, events, pull_input_blocks: *mut AURenderPullInputBlock| -> AUAudioUnitStatus {
			let this = unsafe { &mut *this};
			this.render(flags, timestamp, frame_count, channels, buffers, events, pull_input_blocks.as_ref());
			0
		});
		(*this).internal_render_block = Some(block);
	}
}

struct IVars {
	inputs: Retained<AUAudioUnitBusArray>,
	outputs: Retained<AUAudioUnitBusArray>,
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
	pub struct AudioPlugAudioUnit;

	impl AudioPlugAudioUnit {
		#[unsafe(method_id(initWithComponentDescription))]
		fn initWithComponentDescription_options_error(this: Allocated<Self>, desc: AudioComponentDescription, options: AudioComponentInstantiationOptions, out_error: *mut *mut NSError)
			-> Option<Retained<Self>> 
		{
			//let this = this.set_ivars();

			unsafe { msg_send![super(this), initWithComponentDescription: desc, options: options, error: out_error]}
			/*let this: Option<&mut Self> = unsafe { super(self), initWithComponentDescription: desc, options: options, error: out_error ] } ;
			this.map(|this| {
				let ivars = Box::new(IVars { wrapper: AtomicRefCell::new(Wrapper::new(this.as_super())) });
				let ivar = Self::class().instance_variable(WRAPPER_IVAR_NAME).unwrap();
				unsafe {
					let wrapper_ptr = Box::into_raw(ivars);
					ivar.load_ptr::<*const IVars<P>>(&mut this.superclass)
						.write(wrapper_ptr);
					Wrapper::create_render_block(wrapper_ptr);
					Wrapper::setup_parameter_tree(wrapper_ptr);
				};
				this
			})*/
		}

		#[unsafe(method(inputBusses))]
		fn input_busses(&self) -> &AUAudioUnitBusArray {
			&self.ivars().inputs
		}

		#[unsafe(method(outputBusses))]
		fn output_busses(&self) -> &AUAudioUnitBusArray {
			&self.ivars().outputs
		}

		#[unsafe(method(providesUserInterface))]
		fn provides_user_interface(&self) -> Bool {
			Bool::YES
		}

		/*#[allow(non_snake_case)]
		unsafe extern "C" fn internalRenderBlock(&self, _cmd: Sel) -> &AUInternalRenderBlock {
			self.wrapper().internal_render_block.as_ref().unwrap().deref()
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn maximumFramesToRender(&self, _cmd: Sel) -> AUAudioFrameCount {
			self.wrapper().max_frames_to_render as _
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn setMaximumFramesToRender(&self, _cmd: Sel, maximumFramesToRender: AUAudioFrameCount) {
			self.wrapper_mut().max_frames_to_render = maximumFramesToRender as _;
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn allocateRenderResourcesAndReturnError(&self, _cmd: Sel, error: *mut *mut NSError) -> Bool {
			self.wrapper_mut().allocate_render_resources();
			msg_send![super(self), allocateRenderResourcesAndReturnError: error]
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn deallocateRenderResources(&self, _cmd: Sel) {
			self.wrapper_mut().deallocate_render_resources();
			msg_send![super(self), deallocateRenderResources]
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn channelCapabilities(&self, _cmd: Sel) -> &NSArray<NSNumber> {
			&self.wrapper().channel_capabilities
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn latency(&self, _cmd: Sel) -> NSTimeInterval {
			self.wrapper().plugin.latency_samples() as f64 / DEFAULT_SAMPLE_RATE
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn tailTime(&self, _cmd: Sel) -> NSTimeInterval {
			self.wrapper().plugin.tail_time().as_secs_f64()
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn parameterTree(&self, _cmd: Sel) -> &AUParameterTree {
			&self.wrapper().parameter_tree
		}

		#[allow(non_snake_case)]
		unsafe extern "C" fn supportedViewConfigurations(&self, _cmd: Sel, availableViewConfigurations: &NSArray<AUAudioUnitViewConfiguration>) -> *mut NSIndexSet {
			Retained::into_raw(NSIndexSet::indexSetWithIndexesInRange((0..availableViewConfigurations.count()).into())) 
		}*/
	}
);

impl<P: Plugin + 'static> MyAudioUnit<P> {
	pub fn new_with_component_descriptor_error(desc: AudioComponentDescription, out_error: *mut *mut NSError) -> Retained<Self> {
		unsafe {
			let audio_unit: Retained<Self> = msg_send![
				Self::alloc(),
				initWithComponentDescription: desc,
				error: out_error 
			];

			audio_unit
		}
	}

	pub fn try_with_component_descriptor(desc: AudioComponentDescription) -> Result<Retained<Self>, Retained<NSError>> {
		unsafe {
			msg_send![
				Self::alloc(),
				initWithComponentDescription: desc,
				error: _ 
			]
		}
	}

	pub fn input_busses(&self) -> Retained<AUAudioUnitBusArray> {
		unsafe {
			msg_send![self, inputBusses]
		}
	}

	pub fn output_busses(&self) -> Retained<AUAudioUnitBusArray> {
		unsafe {
			msg_send![self, outputBusses]
		}
	}

	pub fn parameter_tree(&self) -> Retained<AUParameterTree> {
		unsafe {
			msg_send![self, parameterTree]
		}
	}
}

unsafe impl<P: Plugin + 'static> ClassType for MyAudioUnit<P> {
	type Super = AUAudioUnit;
	type ThreadKind = <AUAudioUnit as ClassType>::ThreadKind;

	const NAME: &'static str = P::NAME;

	fn class() -> &'static AnyClass {
		static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

        CLASS.get_or_init(|| {
            let superclass = AUAudioUnit::class();
			let class_name = CString::new(Self::NAME)
				.expect("The objc class name should be a valid c-string");
            let mut builder = ClassBuilder::new(&class_name, superclass).unwrap();

            builder.add_ivar::<*mut IVars<P>>(WRAPPER_IVAR_NAME);
            unsafe {

				builder.add_method(
                    sel!(internalRenderBlock),
                    Self::internalRenderBlock as unsafe extern "C" fn(_, _) -> _,
                );

				builder.add_method(
                    sel!(maximumFramesToRender),
                    Self::maximumFramesToRender as unsafe extern "C" fn(_, _) -> _,
                );

				builder.add_method(
                    sel!(setMaximumFramesToRender:),
                    Self::setMaximumFramesToRender as unsafe extern "C" fn(_, _, _) -> _,
                );
				builder.add_method(
                    sel!(allocateRenderResourcesAndReturnError:),
                    Self::allocateRenderResourcesAndReturnError as unsafe extern "C" fn(_, _, _) -> _,
                );
				builder.add_method(
                    sel!(deallocateRenderResources),
                    Self::deallocateRenderResources as unsafe extern "C" fn(_, _),
                );
				builder.add_method(
                    sel!(channelCapabilities),
                    Self::channelCapabilities as unsafe extern "C" fn(_, _) -> _,
                );
				builder.add_method(
                    sel!(tailTime),
                    Self::tailTime as unsafe extern "C" fn(_, _) -> _,
                );
				builder.add_method(
                    sel!(latency),
                    Self::latency as unsafe extern "C" fn(_, _) -> _,
                );
				builder.add_method(
                    sel!(parameterTree),
                    Self::parameterTree as unsafe extern "C" fn(_, _) -> _,
                );
				builder.add_method(
                    sel!(providesUserInterface),
                    Self::providesUserInterface as unsafe extern "C" fn(_, _) -> _,
                );
				builder.add_method(
                    sel!(supportedViewConfigurations:),
                    Self::supportedViewConfigurations as unsafe extern "C" fn(_, _, _) -> _,
                );
            }

			let cls = builder.register();
			let ivar = cls.instance_variable(WRAPPER_IVAR_NAME).unwrap();
			unsafe { 
				IVAR_OFFSET.write(ivar.offset());
			}
			cls
        })
	}

	fn as_super(&self) -> &AUAudioUnit {
		&self.superclass
	}

	const __INNER: () = ();
	type __SubclassingType = Self;
}
