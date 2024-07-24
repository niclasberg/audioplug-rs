use block2::Block;
use objc2_app_kit::NSViewController;
use objc2_foundation::NSError;
use objc2::mutability::MainThreadOnly;
use objc2::rc::Id;
use objc2::{declare_class, ClassType, DeclaredClass};
use objc2::runtime::NSObjectProtocol;

mod audio_toolbox;
use audio_toolbox::{AUAudioUnit, AUAudioUnitFactory, AudioComponentDescription, AUViewControllerBase};

declare_class!(
	pub struct AUFactory;

	unsafe impl ClassType for AUFactory {
		type Super = NSViewController;
		type Mutability = MainThreadOnly;
		const NAME: &'static str = "AUFactory";
	}

	impl DeclaredClass for AUFactory {
		type Ivars = ();
	}

	unsafe impl AUAudioUnitFactory for AUFactory {
		#[method_id(createAudioUnitWithComponentDescription:error:)]
		fn createAudioUnitWithComponentDescription_error(&self, desc: AudioComponentDescription, error: *mut NSError) -> Id<AUAudioUnit> {
			todo!()
		}

		#[method(requestViewControllerWithCompletionHandler:)]
		fn requestViewControllerWithCompletionHandler(&self, completionHandler: &Block<dyn Fn(*mut AUViewControllerBase)>) {
			//unsafe { completionHandler.call(&self) }
		}
	}

	unsafe impl NSObjectProtocol for AUFactory {}
);