use objc2::rc::Retained;
use objc2_foundation::NSError;
use super::{audio_toolbox::{AUAudioUnit, AudioComponentDescription}, AudioUnit};

pub struct ViewController {

}

#[no_mangle]
pub extern "C" fn create_view_controller() -> *mut ViewController {
	let view_controller = Box::new(ViewController{});
	Box::into_raw(view_controller)
}

#[no_mangle]
pub extern "C" fn destroy_view_controller(view_controller: *mut ViewController) {
	drop(unsafe { Box::from_raw(view_controller) });
}

#[no_mangle]
pub extern "C" fn create_audio_unit(view_controller: *mut ViewController, desc: AudioComponentDescription, error: *mut *mut NSError) -> *mut AUAudioUnit {
	let audio_unit = AudioUnit::new_with_component_descriptor_error(desc, error);
	let audio_unit = Retained::into_super(audio_unit);
	Retained::into_raw(audio_unit) 
}