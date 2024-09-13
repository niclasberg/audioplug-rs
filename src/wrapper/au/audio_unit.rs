use std::{marker::PhantomData, mem::MaybeUninit, ops::Deref, sync::OnceLock};

use block2::RcBlock;
use objc2::{__extern_class_impl_traits, msg_send, msg_send_id, mutability::Mutable, rc::Retained, runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel}, sel, ClassType, Encoding, RefEncode};
use objc2_foundation::{NSArray, NSError, NSIndexSet, NSInteger, NSNumber, NSObject, NSTimeInterval};
use crate::{param::{AnyParameterMap, ParameterId, ParameterMap, PlainValue}, platform::audio_toolbox::{AUParameter, AUValue}, AudioBuffer, Plugin, ProcessContext};
use crate::platform::mac::audio_toolbox::{AUAudioUnitBusArray, AUParameterTree, AUInternalRenderRcBlock, AUAudioUnit, AUAudioUnitBus, AUAudioUnitBusType, AudioUnitRenderActionFlags, AUAudioFrameCount, AURenderEvent, AURenderPullInputBlock, AudioComponentDescription, AUInternalRenderBlock, AudioComponentInstantiationOptions, AUAudioUnitStatus, AUAudioUnitViewConfiguration};
use crate::platform::mac::av_foundation::AVAudioFormat;
use crate::platform::mac::core_audio::{AudioBufferList, AudioTimeStamp};

use super::buffers::BusBuffer;

const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

struct Wrapper<P: Plugin> {
	plugin: P,
    parameters: ParameterMap<P::Parameters>,
	inputs: Retained<AUAudioUnitBusArray>,
	outputs: Retained<AUAudioUnitBusArray>,
	input_buffer: BusBuffer,
	output_buffer: BusBuffer,
	parameter_tree: Retained<AUParameterTree>,
	channel_capabilities: Retained<NSArray<NSNumber>>,
	internal_render_block: Option<AUInternalRenderRcBlock>,
	max_frames_to_render: usize,
	rendering_offline: bool,
}

unsafe impl<P: Plugin> RefEncode for Wrapper<P> {
	const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("?", &[]));
} 

impl<P: Plugin + 'static> Wrapper<P> {
	pub fn new(audio_unit: &mut AUAudioUnit) -> Self {
		let plugin = P::new();
		let parameters = ParameterMap::new(P::Parameters::default());
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
				audio_unit as *mut _, 
				AUAudioUnitBusType::Input,
				&bus_array)
		};

		let output_buffer = BusBuffer::new(2, &format);
		let outputs = unsafe {
			let bus_array = NSArray::from_slice(&[output_buffer.bus()]);
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				audio_unit as *mut _, 
				AUAudioUnitBusType::Output,
				&bus_array)
		};

		let channel_capabilities = NSArray::from_id_slice(&[
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
		_realtime_event_list_head: *const AURenderEvent, 
		_pull_input_block: Option<&AURenderPullInputBlock>
	) {
		let input = AudioBuffer::empty();
        let mut output = AudioBuffer::empty();

        let context = ProcessContext {
            input: &input,
            output: &mut output,
			rendering_offline: self.rendering_offline,
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
				let id = ParameterId::new(p.address() as _);
				if let Some(param_ref) = this.parameters.get_by_id(id) {
					param_ref.internal_set_value_plain(PlainValue::new(value as _));
				}
			});
		(*this).parameter_tree.setImplementorValueObserver(&value_observer);
		let value_provider = 
			RcBlock::new(move |p: *mut AUParameter| -> AUValue {
				let this = unsafe { &*this };
				let p = unsafe { &*p };
				let id = ParameterId::new(p.address() as _);
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

const WRAPPER_IVAR_NAME: &'static str = "wrapper";
static mut IVAR_OFFSET: MaybeUninit<isize> = MaybeUninit::uninit();

__extern_class_impl_traits! {
	unsafe impl (P: Plugin) for MyAudioUnit<P> {
		INHERITS = [AUAudioUnit, NSObject, AnyObject];

		fn as_super(&self) {
			&self.superclass
		}

        fn as_super_mut(&mut self) {
			&mut self.superclass
		}
	}
}


impl<P: Plugin + 'static> MyAudioUnit<P> {
	pub fn new_with_component_descriptor_error(desc: AudioComponentDescription, out_error: *mut *mut NSError) -> Retained<Self> {
		unsafe {
			let audio_unit: Retained<Self> = msg_send_id![
				Self::alloc(),
				initWithComponentDescription: desc,
				error: out_error 
			];

			audio_unit
		}
	}

	pub fn try_with_component_descriptor(desc: AudioComponentDescription) -> Result<Retained<Self>, Retained<NSError>> {
		unsafe {
			msg_send_id![
				Self::alloc(),
				initWithComponentDescription: desc,
				error: _ 
			]
		}
	}

	pub fn input_busses(&self) -> Retained<AUAudioUnitBusArray> {
		unsafe {
			msg_send_id![self, inputBusses]
		}
	}

	pub fn output_busses(&self) -> Retained<AUAudioUnitBusArray> {
		unsafe {
			msg_send_id![self, outputBusses]
		}
	}

	pub fn parameter_tree(&self) -> Retained<AUParameterTree> {
		unsafe {
			msg_send_id![self, parameterTree]
		}
	}

}


impl<P: Plugin + 'static> MyAudioUnit<P> {
	fn wrapper_ptr(&self) -> *const Wrapper<P> {
		let ptr = self as *const _ as *const *const Wrapper<P>;
		unsafe { *ptr.byte_offset(IVAR_OFFSET.assume_init()) }
	}

	fn wrapper_ptr_mut(&mut self) -> *mut Wrapper<P> {
		let ptr = self as *mut _ as *const *mut Wrapper<P>;
		unsafe { *ptr.byte_offset(IVAR_OFFSET.assume_init()) }
	}

	fn wrapper(&self) -> &Wrapper<P> {
		unsafe { &*self.wrapper_ptr() }
	}
	
	fn wrapper_mut(&mut self) -> &mut Wrapper<P> {
		unsafe { &mut *self.wrapper_ptr_mut() }
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn initWithComponentDescription_options_error(
		&mut self,
        _cmd: Sel,
		desc: AudioComponentDescription, 
		options: AudioComponentInstantiationOptions,
		out_error: *mut *mut NSError
	) -> Option<&mut Self> {
		let this: Option<&mut Self> = unsafe { msg_send![super(self), initWithComponentDescription: desc, options: options, error: out_error ] } ;
		this.map(|this| {
			let wrapper = Box::new(Wrapper::new(this.as_super_mut()));
            let ivar = Self::class().instance_variable(WRAPPER_IVAR_NAME).unwrap();
            unsafe {
				let wrapper_ptr = Box::into_raw(wrapper);
                ivar.load_ptr::<*mut Wrapper<P>>(&mut this.superclass)
					.write(wrapper_ptr);
				Wrapper::create_render_block(wrapper_ptr);
				Wrapper::setup_parameter_tree(wrapper_ptr);
            };
            this
        })
	}

	unsafe extern "C" fn dealloc(&mut self, _cmd: Sel) {
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
	unsafe extern "C" fn setMaximumFramesToRender(&mut self, _cmd: Sel, maximumFramesToRender: AUAudioFrameCount) {
		self.wrapper_mut().max_frames_to_render = maximumFramesToRender as _;
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn allocateRenderResourcesAndReturnError(&mut self, _cmd: Sel, error: *mut *mut NSError) -> Bool {
		self.wrapper_mut().allocate_render_resources();
		msg_send![super(self), allocateRenderResourcesAndReturnError: error]
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn deallocateRenderResources(&mut self, _cmd: Sel) {
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

unsafe impl<P: Plugin + 'static> ClassType for MyAudioUnit<P> {
	type Super = AUAudioUnit;
    type Mutability = Mutable;
    const NAME: &'static str = "MyAudioUnit";

    fn class() -> &'static AnyClass {
        static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

        CLASS.get_or_init(|| {
            let superclass = AUAudioUnit::class();
            let mut builder = ClassBuilder::new(Self::NAME, superclass).unwrap();

            builder.add_ivar::<*mut Wrapper<P>>(WRAPPER_IVAR_NAME);
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

    fn as_super(&self) -> &Self::Super {
        &self.superclass
    }

    fn as_super_mut(&mut self) -> &mut Self::Super {
        &mut self.superclass
    }
}
