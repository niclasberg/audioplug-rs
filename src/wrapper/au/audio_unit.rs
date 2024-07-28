use objc2::{declare_class, mutability::InteriorMutable, rc::{Allocated, Retained}, ClassType, DeclaredClass};
use objc2_foundation::{NSError, NSObjectProtocol};
use super::audio_toolbox::{AUAudioUnit, AUInternalRenderBlock, AudioComponentDescription, AudioComponentInstantiationOptions};

declare_class!(
	pub struct AudioUnit;

	unsafe impl ClassType for AudioUnit {
		type Super = AUAudioUnit;
		type Mutability = InteriorMutable;
		const NAME: &'static str = "AudioUnit";
	}

	impl DeclaredClass for AudioUnit {
		type Ivars = ();
	}

	unsafe impl AudioUnit {

		#[method(internalRenderBlock)]
		#[allow(non_snake_case)]
		fn internalRenderBlock(&self) -> *mut AUInternalRenderBlock {
			todo!()
		}		

		#[method_id(initWithComponentDescription:error:)]
		#[allow(non_snake_case)]
		fn initWithComponentDescription_options_error(
			this: Allocated<Self>,
			componentDescription: AudioComponentDescription, 
			options:AudioComponentInstantiationOptions,
			outError: *mut *mut NSError
		) -> Retained<Self> {
			todo!()
		}
	}

	unsafe impl NSObjectProtocol for AudioUnit {}
);

fn aa() {
	
}