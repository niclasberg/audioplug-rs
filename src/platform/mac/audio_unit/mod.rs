use icrate::Foundation::NSError;
use objc2::{extern_class, mutability, ClassType, runtime::{NSObject, NSObjectProtocol}, extern_methods, rc::{Id, Allocated}};

mod audio_component;
mod audio_unit_factory;
pub use audio_component::AudioComponentDescription;
pub use audio_unit_factory::*;

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct AUAudioUnit;

	unsafe impl ClassType for AUAudioUnit {
		type Super = NSObject;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl AUAudioUnit {
		#[method_id(initWithComponentDescription:error:)]
		#[allow(non_snake_case)]
		pub unsafe fn initWithComponentDescription_error(
			this: Allocated<Self>,
			componentDescription: AudioComponentDescription, 
			outError: *mut NSError
		) -> Id<Self>;


	}
);