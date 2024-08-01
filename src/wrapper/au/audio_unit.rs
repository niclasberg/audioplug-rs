use std::{cell::RefCell, collections::HashMap, marker::PhantomData, sync::Once};

use objc2::{declare_class, msg_send_id, mutability::{InteriorMutable,Mutable}, rc::{Allocated, Retained}, runtime::{AnyClass, ClassBuilder}, ClassType, DeclaredClass, Encode, Encoding, Message, RefEncode};
use objc2_foundation::{NSError, NSObject, NSObjectProtocol};
use crate::{param::{ParameterGetter, ParameterId}, AudioLayout, Plugin};

use super::audio_toolbox::{AUAudioUnit, AUAudioUnitBusArray, AUAudioUnitBusType, AUInternalRenderBlock, AudioComponentDescription, AudioComponentInstantiationOptions};

pub struct Wrapper<P: Plugin> {
	plugin: RefCell<P>,
    parameters: P::Parameters,
	parameter_getters: HashMap<ParameterId, ParameterGetter<P::Parameters>>
}

unsafe impl<P: Plugin> RefEncode for Wrapper<P> {
	const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("?", &[]));
} 

impl<P: Plugin> Wrapper<P> {
	pub fn input_busses(&self) -> *mut AUAudioUnitBusArray {
		todo!()
	}

	pub fn output_busses(&self) -> *mut AUAudioUnitBusArray {
		todo!()
	}
}

#[repr(C)]
struct MyAudioUnit<P: Plugin> {
    // Required to give MyObject the proper layout
    superclass: AUAudioUnit,
    p: PhantomData<P>,
}

unsafe impl<P: Plugin> RefEncode for MyAudioUnit<P> {
    const ENCODING_REF: Encoding = AUAudioUnit::ENCODING_REF;
}

unsafe impl<P: Plugin> Message for MyAudioUnit<P> {}

unsafe impl<P: Plugin> ClassType for MyAudioUnit<P> {
	type Super = NSObject;
    type Mutability = Mutable;
    const NAME: &'static str = "MyAudioUnit";

    fn class() -> &'static AnyClass {
        // TODO: Use std::lazy::LazyCell
        static REGISTER_CLASS: Once = Once::new();

        REGISTER_CLASS.call_once(|| {
            let superclass = NSObject::class();
            let mut builder = ClassBuilder::new(Self::NAME, superclass).unwrap();

            builder.add_ivar::<*mut Wrapper<P>>("wrapper");
            /*unsafe {
                builder.add_method(
                    sel!(initWithPtr:),
                    Self::init_with_ptr as unsafe extern "C" fn(_, _, _) -> _,
                );
            }*/


            let _cls = builder.register();
        });

        AnyClass::get(Self::NAME).unwrap()
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

fn create_busses(audio_layout: &AudioLayout) -> (Retained<AUAudioUnitBusArray>, Retained<AUAudioUnitBusArray>) {


	todo!()
}