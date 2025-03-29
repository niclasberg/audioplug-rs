use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use super::MyAudioUnit;
use crate::{
    app::{AppState, HostHandle, MyHandler},
    param::{NormalizedValue, ParameterId, ParameterInfo, ParameterMap, Params, PlainValue},
    platform::{mac::dispatch::create_block_dispatching_to_main2, view::View},
    Editor, Plugin,
};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_audio_toolbox::{
    AUAudioUnit, AUParameterAddress, AUParameterAutomationEventType, AUParameterObserverToken,
    AUParameterTree, AUValue, AudioComponentDescription,
};
use objc2_core_foundation::CGSize;
use objc2_foundation::{MainThreadMarker, NSError};

struct AUV3HostHandle {
    parameter_tree: Retained<AUParameterTree>,
    observer_token: AUParameterObserverToken,
}

impl HostHandle for AUV3HostHandle {
    fn begin_edit(&self, id: ParameterId) {
        if let Some(parameter) = unsafe { self.parameter_tree.parameterWithAddress(id.into()) } {
            unsafe {
                let value = parameter.value();
                parameter.setValue_originator_atHostTime_eventType(
                    value,
                    self.observer_token,
                    0,
                    AUParameterAutomationEventType::Touch,
                );
            }
        }
    }

    fn end_edit(&self, id: ParameterId) {
        if let Some(parameter) = unsafe { self.parameter_tree.parameterWithAddress(id.into()) } {
            unsafe {
                let value = parameter.value();
                parameter.setValue_originator_atHostTime_eventType(
                    value,
                    self.observer_token,
                    0,
                    AUParameterAutomationEventType::Release,
                );
            }
        }
    }

    fn perform_edit(&self, info: &dyn ParameterInfo, value: NormalizedValue) {
        if let Some(parameter) =
            unsafe { self.parameter_tree.parameterWithAddress(info.id().into()) }
        {
            let plain_value = info.denormalize(value);
            unsafe {
                parameter
                    .setValue_originator(Into::<f64>::into(plain_value) as _, self.observer_token)
            }
        }
    }
}

// In order for the compiled app extension to have the correct binary format, we have to compile it with
// clang (it needs the _NSExtensionMain instead of a regular main function).
// We therefore implement the actual viewcontroller class in objective C and expose a small c api
// that the view controller interacts with.
pub struct ViewController<P: Plugin> {
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<P::Editor>>,
    parameters: Rc<ParameterMap<P::Parameters>>,
    parameter_observer: RcBlock<dyn Fn(AUParameterAddress, AUValue)>,
    _phantom: PhantomData<P>,
}

impl<P: Plugin + 'static> ViewController<P> {
    pub fn new() -> Self {
        let parameters = ParameterMap::new(P::Parameters::new());
        let app_state = Rc::new(RefCell::new(AppState::new(parameters.clone())));
        let editor = Rc::new(RefCell::new(P::Editor::new()));
        let parameter_observer = {
            let weak_app_state = Rc::downgrade(&app_state);
            create_block_dispatching_to_main2(
                MainThreadMarker::new().unwrap(),
                move |address, value| {
                    let id = ParameterId(address as _);
                    let value = PlainValue::new(value as _);
                    if let Some(app_state) = weak_app_state.upgrade() {
                        app_state
                            .borrow_mut()
                            .set_plain_parameter_value_from_host(id, value);
                    }
                },
            )
        };

        Self {
            app_state,
            editor,
            parameter_observer,
            parameters,
            _phantom: PhantomData,
        }
    }

    pub fn create_audio_unit(
        &mut self,
        desc: AudioComponentDescription,
        error: *mut *mut NSError,
    ) -> *mut AUAudioUnit {
        let plugin = P::new();
        let Some(audio_unit) =
            MyAudioUnit::new_with_component_descriptor_error(plugin, desc, error)
        else {
            return std::ptr::null_mut();
        };

        {
            let parameter_tree = unsafe { audio_unit.parameterTree() }.unwrap();
            let param_token = unsafe {
                parameter_tree
                    .tokenByAddingParameterObserver(RcBlock::as_ptr(&self.parameter_observer))
            };
            let handle = AUV3HostHandle {
                observer_token: param_token,
                parameter_tree: parameter_tree.clone(),
            };
            let mut app_state = self.app_state.borrow_mut();
            app_state.set_host_handle(Some(Box::new(handle)));
        }

        Retained::into_raw(Retained::into_super(audio_unit))
    }

    pub fn create_view(&mut self) -> *mut NSView {
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let app_state = self.app_state.clone();

        let view = {
            let editor = RefCell::borrow_mut(&self.editor);
            editor.view(self.parameters.parameters_ref())
        };
        let handler = MyHandler::new(app_state, view);
        let view = View::new(mtm, handler, None);

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
