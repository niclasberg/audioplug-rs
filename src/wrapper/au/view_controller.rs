use std::{cell::RefCell, ffi::c_void, marker::PhantomData, rc::Rc};

use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_foundation::{MainThreadMarker, NSError};
use crate::{app::AppState, platform::view::View, window::MyHandler, Editor, Plugin};

use super::{audio_toolbox::{AUAudioUnit, AudioComponentDescription}, MyAudioUnit};

pub struct ViewController<P: Plugin> {
	app_state: Rc<RefCell<AppState>>,
	editor: Rc<RefCell<P::Editor>>,
	_phantom: PhantomData<P>
}

impl<P: Plugin + 'static> ViewController<P> {
	pub fn new() -> Self {
		let app_state =  Rc::new(RefCell::new(AppState::new(())));
		let editor = Rc::new(RefCell::new(P::Editor::new()));
		Self {
			app_state,
			editor,
			_phantom: PhantomData
		}
	}

	pub fn create_audio_unit(&mut self, desc: AudioComponentDescription, error: *mut *mut NSError) -> *mut AUAudioUnit {
		let audio_unit = MyAudioUnit::<P>::new_with_component_descriptor_error(desc, error);
		let audio_unit = Retained::into_super(audio_unit);
		Retained::into_raw(audio_unit) 
	}

	pub fn create_view(&mut self) -> *mut NSView {
		let mtm = MainThreadMarker::new().unwrap();
		let app_state = self.app_state.clone();

		let _editor = self.editor.clone();
		let handler = MyHandler::new(app_state, move |ctx| {
			let editor = RefCell::borrow(&_editor);
			let params = P::Parameters::default();
			editor.view(ctx, &params)
		});
		let view = View::new(mtm, handler);
		Retained::into_raw(Retained::into_super(view))
	}
}