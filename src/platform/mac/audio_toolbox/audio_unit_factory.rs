use block2::Block;
use objc2_foundation::{NSError, NSExtensionRequestHandling};
use objc2_app_kit::NSViewController;
use objc2::{extern_protocol, rc::Id, ProtocolType};

use super::{AudioComponentDescription, AUAudioUnit};

pub type AUViewControllerBase = NSViewController;

extern_protocol!(
	pub unsafe trait AUAudioUnitFactory: NSExtensionRequestHandling {
		#[unsafe(method(createAudioUnitWithComponentDescription:error:))]
		#[allow(non_snake_case)]
		unsafe fn createAudioUnitWithComponentDescription_error(&self, desc: AudioComponentDescription, error: *mut *mut NSError) -> Id<AUAudioUnit>;

		#[unsafe(method(requestViewControllerWithCompletionHandler:))]
		#[allow(non_snake_case)]
		unsafe fn requestViewControllerWithCompletionHandler(&self, completionHandler: &Block<dyn Fn(*mut AUViewControllerBase)>);
	}
);