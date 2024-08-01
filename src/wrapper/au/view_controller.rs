use std::{cell::RefCell, rc::Rc};

use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_foundation::{MainThreadMarker, NSError};
use crate::{app::AppState, core::{Color, Point, Rectangle, Size}, platform::view::View, view::Fill, window::MyHandler};

use super::{audio_toolbox::{AUAudioUnit, AudioComponentDescription}, AudioUnit};

pub struct ViewController {
	app_state: Rc<RefCell<AppState>>
}

#[no_mangle]
pub extern "C" fn create_view_controller() -> *mut ViewController {
	let app_state =  Rc::new(RefCell::new(AppState::new(())));
	let view_controller = Box::new(ViewController{
		app_state
	});
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

#[no_mangle]
pub extern "C" fn create_view(view_controller: *mut ViewController) -> *mut NSView {
	let mtm = MainThreadMarker::new().unwrap();
	let app_state = unsafe { &*view_controller }.app_state.clone();
	let handler = MyHandler::new(app_state, |_| 
		Rectangle::new(Point::ZERO, Size::new(20.0, 20.0)).fill(Color::BLACK));
	let view = View::new(mtm, handler);
	Retained::into_raw(Retained::into_super(view))
}