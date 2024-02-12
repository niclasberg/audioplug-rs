use block2::Block;
use icrate::{Foundation::NSError, AppKit::NSViewController};
use objc2::{extern_protocol, rc::Id, ProtocolType};

use super::{AudioComponentDescription, AUAudioUnit};

pub type AUViewControllerBase = NSViewController;

extern_protocol!(
	pub unsafe trait AUAudioUnitFactory {
		#[method_id(createAudioUnitWithComponentDescription:error:)]
		fn createAudioUnitWithComponentDescription_error(&self, desc: AudioComponentDescription, error: *mut NSError) -> Id<AUAudioUnit>;

		#[method(requestViewControllerWithCompletionHandler:)]
		fn requestViewControllerWithCompletionHandler(&self, completionHandler: &Block<(*mut AUViewControllerBase, ), ()>);
	}

	unsafe impl ProtocolType for dyn AUAudioUnitFactory {}
);