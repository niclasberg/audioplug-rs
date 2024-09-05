use std::{cell::RefCell, clone, marker::PhantomData, rc::{Rc, Weak}, sync::atomic::AtomicPtr};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_foundation::{CGSize, MainThreadMarker, NSError};
use crate::{app::{AppState, HostHandle, MyHandler}, param::{ParameterId, PlainValue}, platform::{audio_toolbox::{AUParameterAddress, AUValue}, dispatch::create_block_dispatching_to_main2, view::View, DispatchQueue, MainThreadQueue}, Editor, Plugin};
use crate::platform::mac::audio_toolbox::{AUAudioUnit, AudioComponentDescription};
use super::MyAudioUnit;

struct AUV3HostHandle;

impl HostHandle for AUV3HostHandle {
	fn begin_edit(&self, _id: crate::param::ParameterId) {
		
	}

	fn end_edit(&self, _id: crate::param::ParameterId) {
		
	}

	fn perform_edit(&self, _id: crate::param::ParameterId, _value: crate::param::NormalizedValue) {
		
	}
}

// In order for the compiled app extension to have the correct binary format, we have to compile it with
// clang (it needs the _NSExtensionMain instead of a regular main function).
// We therefore implement the actual viewcontroller class in objective C and expose a small c api
// that the view controller interacts with.
pub struct ViewController<P: Plugin> {
	app_state: Rc<RefCell<AppState>>,
	editor: Rc<RefCell<P::Editor>>,
	parameter_observer: RcBlock<dyn Fn(AUParameterAddress, AUValue)>,
	_phantom: PhantomData<P>
}

impl<P: Plugin + 'static> ViewController<P> {
	pub fn new() -> Self {
		let app_state =  Rc::new(RefCell::new(AppState::new(P::Parameters::default(), AUV3HostHandle)));
		let editor = Rc::new(RefCell::new(P::Editor::new()));
		let parameter_observer = {
			let weak_app_state = Rc::downgrade(&app_state);
			create_block_dispatching_to_main2(MainThreadMarker::new().unwrap(), move |address, value| {
				let id = ParameterId::new(address as _);
				let value = PlainValue::new(value as _);
				if let Some(app_state) = weak_app_state.upgrade() {
					app_state.borrow_mut().set_plain_parameter_value_from_host(id, value);
				}
			})
		};
		
		Self {
			app_state,
			editor,
			parameter_observer,
			_phantom: PhantomData
		}
	}

	pub fn create_audio_unit(&mut self, desc: AudioComponentDescription, error: *mut *mut NSError) -> *mut AUAudioUnit {
		let audio_unit = MyAudioUnit::<P>::new_with_component_descriptor_error(desc, error);

		{
			let parameter_tree = audio_unit.parameter_tree();
			// The observer block can be called on any thread. We downcast to weak here,
			// and then we ensure that we upgrade to an Rc on the main thread by dispatching.
			


		}
		

		let audio_unit = Retained::into_super(audio_unit);
		Retained::into_raw(audio_unit) 
	}

	pub fn create_view(&mut self) -> *mut NSView {
		let mtm = unsafe { MainThreadMarker::new_unchecked() };
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

	pub fn preferred_size(&self) -> CGSize {
		if let Some(size) = self.editor.borrow().prefered_size() {
			size.into()
		} else {
			CGSize::new(520.0, 480.0)
		}
	}
}