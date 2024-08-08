use std::{cell::{OnceCell, RefCell}, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ops::Deref, sync::{Once, OnceLock}};

use block2::{RcBlock, StackBlock};
use objc2::{__extern_class_impl_traits, declare_class, msg_send, msg_send_id, mutability::{InteriorMutable,Mutable}, rc::{Allocated, PartialInit, Retained}, runtime::{AnyClass, AnyObject, ClassBuilder, Ivar, Sel}, sel, ClassType, DeclaredClass, Encode, Encoding, Message, RefEncode};
use objc2_foundation::{NSArray, NSError, NSInteger, NSObject, NSObjectProtocol};
use crate::{param::{ParameterGetter, ParameterId, Params}, platform::core_audio::{AudioBufferList, AudioTimeStamp}, AudioLayout, Plugin};

use super::{audio_toolbox::{AUAudioFrameCount, AUAudioUnit, AUAudioUnitBus, AUAudioUnitBusArray, AUAudioUnitBusType, AUAudioUnitStatus, AUInternalRenderBlock, AUInternalRenderRcBlock, AURenderEvent, AURenderPullInputBlock, AudioComponentDescription, AudioComponentInstantiationOptions, AudioUnitRenderActionFlags}, av_foundation::AVAudioFormat};

struct Wrapper<P: Plugin> {
	plugin: RefCell<P>,
    parameters: P::Parameters,
	parameter_getters: HashMap<ParameterId, ParameterGetter<P::Parameters>>,
	inputs: Retained<AUAudioUnitBusArray>,
	outputs: Retained<AUAudioUnitBusArray>,
	block: Option<AUInternalRenderRcBlock>
}

unsafe impl<P: Plugin> RefEncode for Wrapper<P> {
	const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("?", &[]));
} 

impl<P: Plugin> Wrapper<P> {
	pub fn new(audio_unit: &mut AUAudioUnit) -> Self {
		let plugin = RefCell::new(P::new());
		let parameters = P::Parameters::default();
		let parameter_getters = P::Parameters::PARAMS.iter()
			.map(|getter| (getter(&parameters).id(), *getter))
			.collect();

		let format = unsafe {
			AVAudioFormat::initStandardFormatWithSampleRate_channels(
				AVAudioFormat::alloc(), 
				44100.0, 
				2)
		};

		let inputs = unsafe {
			let input_bus = AUAudioUnitBus::initWithFormat_error(
				AUAudioUnitBus::alloc(),
				&format).unwrap();
			let bus_array = NSArray::from_slice(&[input_bus.as_ref()]);
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				audio_unit as *mut _, 
				AUAudioUnitBusType::Input,
				&bus_array)
		};
		let outputs = unsafe {
			let output_bus = AUAudioUnitBus::initWithFormat_error(
				AUAudioUnitBus::alloc(),
				&format).unwrap();
			let bus_array = NSArray::from_slice(&[output_bus.as_ref()]);
			AUAudioUnitBusArray::initWithAudioUnit_busType_busses(
				AUAudioUnitBusArray::alloc(), 
				audio_unit as *mut _, 
				AUAudioUnitBusType::Output,
				&bus_array)
		};
		
		Self {
			plugin,
			parameters,
			parameter_getters,
			inputs,
			outputs,
			block: None
		}
	}

	pub fn input_busses(&self) -> *mut AUAudioUnitBusArray {
		Retained::into_raw(self.inputs.clone())
	}

	pub fn output_busses(&self) -> *mut AUAudioUnitBusArray {
		Retained::into_raw(self.outputs.clone())
	}

	pub fn render(&mut self, 
		flags: *mut AudioUnitRenderActionFlags, 
		timestamp: *const AudioTimeStamp, 
		frame_count: AUAudioFrameCount, 
		output_bus_bumber: NSInteger, 
		output_data: *mut AudioBufferList, 
		realtime_event_list_head: *const AURenderEvent, 
		pull_input_block: *mut AURenderPullInputBlock
	) {
		
	}

}

#[repr(C)]
pub struct MyAudioUnit<P: Plugin + 'static> {
    // Required to give MyObject the proper layout
    superclass: AUAudioUnit,
    p: PhantomData<P>,
}

const WRAPPER_IVAR_NAME: &'static str = "wrapper";

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

	fn ivar_offset() -> isize {
		thread_local! {
			pub static IVAR_OFFSET: OnceCell<isize> = OnceCell::new();
		}
		
		IVAR_OFFSET.with(move |cell| {
			*cell.get_or_init(|| {
				let cls = MyAudioUnit::<P>::class();
				let ivar = cls.instance_variable(WRAPPER_IVAR_NAME).unwrap();
				ivar.offset()
			})
		})
	}

	fn wrapper(&self) -> &Wrapper<P> {
		let ptr = self as *const _ as *const *const Wrapper<P>;
		let ivar_ptr = unsafe { *ptr.byte_offset(Self::ivar_offset()) };
		unsafe { &*ivar_ptr }
	}
	
	fn wrapper_mut(&mut self) -> &mut Wrapper<P> {
		let ptr = self as *mut _ as *const *mut Wrapper<P>;
		let ivar_ptr = unsafe { *ptr.byte_offset(Self::ivar_offset()) };
		unsafe { &mut *ivar_ptr }
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

				let wrapper_ptr = wrapper_ptr;
				let block: AUInternalRenderRcBlock = RcBlock::new(move |flags, timestamp, frame_count, channels, buffers, events, pull_input_blocks| -> AUAudioUnitStatus {
					let wrapper = unsafe { &mut *wrapper_ptr };
					wrapper.render(flags, timestamp, frame_count, channels, buffers, events, pull_input_blocks);
					0
				});
				(&mut *wrapper_ptr).block = Some(block);
				
            };
            this
        })
	}

	unsafe extern "C" fn dealloc(&mut self, _cmd: Sel) {
		let ivar = Self::class().instance_variable(WRAPPER_IVAR_NAME).unwrap();
		let wrapper_ptr = ivar.load_ptr::<*mut Wrapper<P>>(&mut self.superclass);
		std::mem::drop(Box::from_raw(*wrapper_ptr));
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn inputBusses(&self, _cmd: Sel) -> *mut AUAudioUnitBusArray {
		self.wrapper().input_busses()
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn outputBusses(&self, _cmd: Sel) -> *mut AUAudioUnitBusArray {
		self.wrapper().output_busses()
	}

	#[allow(non_snake_case)]
	unsafe extern "C" fn internalRenderBlock(&mut self, _cmd: Sel) -> *mut AUInternalRenderBlock {
		std::mem::transmute(self.wrapper_mut().block.clone().unwrap())
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
            }

            builder.register()
        })
    }

    fn as_super(&self) -> &Self::Super {
        &self.superclass
    }

    fn as_super_mut(&mut self) -> &mut Self::Super {
        &mut self.superclass
    }
}




pub struct IVars {
	input_busses: RefCell<Option<Retained<AUAudioUnitBusArray>>>,
	output_busses: RefCell<Option<Retained<AUAudioUnitBusArray>>>
}

declare_class!(
	pub struct AudioUnit;

	unsafe impl ClassType for AudioUnit {
		type Super = AUAudioUnit;
		type Mutability = InteriorMutable;
		const NAME: &'static str = "AudioUnit";
	}

	impl DeclaredClass for AudioUnit {
		type Ivars = IVars;
	}

	unsafe impl AudioUnit {
		#[method(internalRenderBlock)]
		#[allow(non_snake_case)]
		fn internalRenderBlock(&self) -> *mut AUInternalRenderBlock {
			std::ptr::null_mut()
		}	

		#[method_id(inputBusses)]
		#[allow(non_snake_case)]
		fn inputBusses(&self) -> Option<Retained<AUAudioUnitBusArray>> {
			let input_busses = self.ivars().input_busses.borrow();
			input_busses.clone()
		}

		#[method_id(outputBusses)]
		#[allow(non_snake_case)]
		fn outputBusses(&self) -> Option<Retained<AUAudioUnitBusArray>> {
			let output_busses = self.ivars().output_busses.borrow();
			output_busses.clone()
		}

		#[method_id(initWithComponentDescription:options:error:)]
		#[allow(non_snake_case)]
		fn initWithComponentDescription_options_error(
			this: Allocated<Self>,
			desc: AudioComponentDescription, 
			options: AudioComponentInstantiationOptions,
			out_error: *mut *mut NSError
		) -> Retained<Self> {
			let ivars = IVars {
				input_busses: RefCell::new(None),
				output_busses: RefCell::new(None),
			};
			let this = this.set_ivars(ivars);
			unsafe { msg_send_id![super(this), initWithComponentDescription: desc, options: options, error: out_error ] } 
		}
	}

	unsafe impl NSObjectProtocol for AudioUnit {}
);

impl AudioUnit {
	pub fn new_with_component_descriptor_error(desc: AudioComponentDescription, out_error: *mut *mut NSError) -> Retained<Self> {
		unsafe {
			let this = AudioUnit::alloc();
			let audio_unit: Retained<Self> = msg_send_id![
				this,
				initWithComponentDescription: desc,
				error: out_error 
			];

			audio_unit
		}
	}
}

fn create_busses(audio_unit: *const AUAudioUnit, audio_layout: &AudioLayout) -> (Retained<AUAudioUnitBusArray>, Retained<AUAudioUnitBusArray>) {

	todo!()
}