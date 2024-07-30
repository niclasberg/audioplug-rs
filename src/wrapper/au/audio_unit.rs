use std::cell::RefCell;

use objc2::{declare_class, msg_send_id, mutability::InteriorMutable, rc::{Allocated, Retained}, ClassType, DeclaredClass};
use objc2_foundation::{NSError, NSObjectProtocol};
use super::audio_toolbox::{AUAudioUnit, AUAudioUnitBusArray, AUInternalRenderBlock, AudioComponentDescription, AudioComponentInstantiationOptions};

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