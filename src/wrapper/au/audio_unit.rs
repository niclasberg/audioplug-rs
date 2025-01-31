use std::{ffi::CStr, marker::PhantomData, mem::MaybeUninit, ops::Deref, rc::Rc, sync::OnceLock};

use atomic_refcell::AtomicRefCell;
use block2::RcBlock;
use objc2::{define_class, msg_send, rc::Retained, runtime::{AnyClass, Bool, ClassBuilder, Sel}, sel, AllocAnyThread, ClassType, Encoding, Message, RefEncode};
use objc2_foundation::{NSArray, NSError, NSIndexSet, NSInteger, NSNumber, NSObject, NSTimeInterval};
use crate::{param::{AnyParameterMap, ParameterId, ParameterMap, Params, PlainValue}, platform::{audio_toolbox::{AUAudioFrameCount, AUAudioUnit, AUAudioUnitBusArray, AUAudioUnitBusType, AUAudioUnitStatus, AUAudioUnitViewConfiguration, AUInternalRenderBlock, AUInternalRenderRcBlock, AUParameter, AUParameterTree, AURenderEvent, AURenderEventType, AURenderPullInputBlock, AUValue, AudioComponentInstantiationOptions, AudioUnitRenderActionFlags}, av_foundation::AVAudioFormat}, AudioBuffer, Plugin, ProcessContext};
use crate::platform::mac::core_audio::{AudioBufferList, AudioTimeStamp};

use super::{buffers::BusBuffer, AudioComponentDescription};

const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

trait Wrapper {
	fn allocate_render_resources(&mut self);
	fn deallocate_render_resources(&mut self);
	unsafe fn setup_parameter_tree(&mut self);
}

struct WrapperImpl<P: Plugin> {
	plugin: P,
    parameters: Rc<ParameterMap<P::Parameters>>,
	inputs: Retained<AUAudioUnitBusArray>,
	outputs: Retained<AUAudioUnitBusArray>,
	input_buffer: BusBuffer,
	output_buffer: BusBuffer,
	parameter_tree: Retained<AUParameterTree>,
	channel_capabilities: Retained<NSArray<NSNumber>>,
	internal_render_block: Option<AUInternalRenderRcBlock>,
	max_frames_to_render: usize,
	rendering_offline: bool
}

unsafe impl<P: Plugin> RefEncode for WrapperImpl<P> {
	const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("?", &[]));
} 

impl<P: Plugin + 'static> WrapperImpl<P> {
	pub fn new(audio_unit: &AUAudioUnit) -> Self {
		let plugin = P::new();
		let parameters = ParameterMap::new(P::Parameters::new());
		let parameter_tree = super::utils::create_parameter_tree(&parameters);

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
			inputs,
			outputs,
			input_buffer,
			output_buffer,
			parameter_tree,
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

	pub unsafe fn setup_parameter_tree(this: *const Self) {
		let value_observer =
			RcBlock::new(move |p: *mut AUParameter, value: AUValue| {
				let this = unsafe { &*this };
				let p = unsafe { &*p };
				let id = ParameterId(p.address() as _);
				if let Some(param_ref) = this.parameters.get_by_id(id) {
					param_ref.set_value_plain(PlainValue::new(value as _));
				}
			});
		(*this).parameter_tree.setImplementorValueObserver(&value_observer);
		let value_provider = 
			RcBlock::new(move |p: *mut AUParameter| -> AUValue {
				let this = unsafe { &*this };
				let p = unsafe { &*p };
				let id = ParameterId(p.address() as _);
				this.parameters.get_by_id(id).map_or(0.0, |param| {
					let value: f64 = param.plain_value().into();
					value as _
				})
			});
		(*this).parameter_tree.setImplementorValueProvider(&value_provider);
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

#[repr(C)]
pub struct MyAudioUnit<P: Plugin + 'static> {
    // Required to give MyObject the proper layout
    superclass: AUAudioUnit,
    p: PhantomData<P>,
}

struct IVars {
	wrapper: AtomicRefCell<Box<dyn Wrapper>>
}

define_class!(
	#[unsafe(super(AUAudioUnit))]
	#[name = "AudioPlugAudioUnit"]
	#[ivars = IVars]
	pub struct AudioPlugAudioUnit;
);

const CLASS_NAME: &'static CStr = c"AudioPlugUnit";
const WRAPPER_IVAR_NAME: &'static CStr = c"wrapper";
static mut IVAR_OFFSET: MaybeUninit<isize> = MaybeUninit::uninit();

unsafe impl<P: Plugin + 'static> RefEncode for MyAudioUnit<P> {
    const ENCODING_REF: Encoding = NSObject::ENCODING_REF;
}

unsafe impl<P: Plugin + 'static> Message for MyAudioUnit<P> {}

impl<P: Plugin + 'static> MyAudioUnit<P> {
	pub fn new_with_component_descriptor_error(desc: AudioComponentDescription, out_error: *mut *mut NSError) -> Retained<Self> {
		unsafe {
			let this = msg_send![Self::class(), new];
			let audio_unit: Retained<Self> = msg_send![
				Self::alloc(),
				initWithComponentDescription: desc,
				error: out_error 
			];

			audio_unit
		}
	}

	pub fn as_super(&self) -> &AUAudioUnit {
		&self.superclass
	}

	pub fn as_super_mut(&mut self) -> &mut AUAudioUnit {
		&mut self.superclass
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

impl<P: Plugin + 'static> MyAudioUnit<P> {
	pub fn class() -> &'static AnyClass {
        static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

        CLASS.get_or_init(|| {
            let superclass = AUAudioUnit::class();
            let mut builder = ClassBuilder::new(CLASS_NAME, superclass).unwrap();

            builder.add_ivar::<*mut WrapperImpl<P>>(WRAPPER_IVAR_NAME);
            unsafe {
                builder.add_method(
                    sel!(initWithComponentDescription:options:error:),
                    Self::initWithComponentDescription_options_error as unsafe extern "C" fn(_, _, _, _, _) -> _,
                );
				builder.add_method(
                    sel!(dealloc),
                    Self::dealloc as unsafe extern "C" fn(_, _),
                );
				builder.add_method(
                    sel!(inputBusses),
                    Self::inputBusses as unsafe extern "C" fn(_, _) -> _,
                );
				builder.add_method(
                    sel!(outputBusses),
                    Self::outputBusses as unsafe extern "C" fn(_, _) -> _,
                );

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

	fn ivars_ptr(&self) -> *const IVars<P> {
		let ptr = self as *const _ as *const *const IVars<P>;
		unsafe { *ptr.byte_offset(IVAR_OFFSET.assume_init()) }
	}

	fn wrapper(&self) -> &WrapperImpl<P> {
		unsafe { &*self.wrapper_ptr() }
	}
	
	fn wrapper_mut(&mut self) -> &mut WrapperImpl<P> {
		unsafe { &mut *self.wrapper_ptr_mut() }
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn initWithComponentDescription_options_error(
		&self,
        _cmd: Sel,
		desc: AudioComponentDescription, 
		options: AudioComponentInstantiationOptions,
		out_error: *mut *mut NSError
	) -> Option<&mut Self> {
		let this: Option<&mut Self> = unsafe { msg_send![self.as_super(), initWithComponentDescription: desc, options: options, error: out_error ] } ;
		this.map(|this| {
			let wrapper = Box::new(WrapperImpl::new(this.as_super_mut()));
            let ivar = Self::class().instance_variable(WRAPPER_IVAR_NAME).unwrap();
            unsafe {
				let wrapper_ptr = Box::into_raw(wrapper);
                ivar.load_ptr::<*mut WrapperImpl<P>>(&mut this.superclass)
					.write(wrapper_ptr);
				WrapperImpl::create_render_block(wrapper_ptr);
				WrapperImpl::setup_parameter_tree(wrapper_ptr);
            };
            this
        })
	}

	unsafe extern "C" fn dealloc(&self, _cmd: Sel) {
		std::mem::drop(Box::from_raw(self.wrapper_ptr_mut()));
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn inputBusses(&self, _cmd: Sel) -> &AUAudioUnitBusArray {
		&self.wrapper().inputs
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn outputBusses(&self, _cmd: Sel) -> &AUAudioUnitBusArray {
		&self.wrapper().outputs
	}

	#[allow(non_snake_case)]
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
	unsafe extern "C" fn providesUserInterface(&self, _cmd: Sel) -> Bool {
		Bool::YES
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn supportedViewConfigurations(&self, _cmd: Sel, availableViewConfigurations: &NSArray<AUAudioUnitViewConfiguration>) -> *mut NSIndexSet {
		Retained::into_raw(NSIndexSet::indexSetWithIndexesInRange((0..availableViewConfigurations.count()).into())) 
	}
}
