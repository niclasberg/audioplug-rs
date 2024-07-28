use block2::Block;
use objc2_foundation::NSError;
use objc2_app_kit::NSViewController;
use objc2::{extern_protocol, rc::Id, ProtocolType};

use super::{AudioComponentDescription, AUAudioUnit};

pub type AUViewControllerBase = NSViewController;

extern_protocol!(
	pub unsafe trait AUAudioUnitFactory {
		#[method_id(createAudioUnitWithComponentDescription:error:)]
		unsafe fn createAudioUnitWithComponentDescription_error(&self, desc: AudioComponentDescription, error: *mut *mut NSError) -> Id<AUAudioUnit>;

		#[method(requestViewControllerWithCompletionHandler:)]
		unsafe fn requestViewControllerWithCompletionHandler(&self, completionHandler: &Block<dyn Fn(*mut AUViewControllerBase)>);
	}

	unsafe impl ProtocolType for dyn AUAudioUnitFactory {}
);