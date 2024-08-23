use std::cell::{OnceCell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;

use vst3_com::VstPtr;
use vst3_com::vst::{kRootUnitId, ParameterFlags};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::{IID, VST3, c_void};
use vst3_sys::vst::{IEditController, ParameterInfo, IComponentHandler};

use vst3_sys as vst3_com;

use crate::app::{AppState, HostHandle};
use crate::param::{NormalizedValue, ParamRef, ParameterId, Params, PlainValue};
use crate::Editor;

use super::plugview::PlugView;
use super::util::strcpyw;

struct VST3HostHandle {
    component_handler: VstPtr<dyn IComponentHandler>,
}

impl HostHandle for VST3HostHandle {
    fn begin_edit(&self, id: ParameterId) {
        unsafe { self.component_handler.begin_edit(id.into()) };
    }

    fn end_edit(&self, id: ParameterId) {
        unsafe { self.component_handler.end_edit(id.into()) };
    }

    fn perform_edit(&self, id: ParameterId, value: NormalizedValue) {
        unsafe { self.component_handler.perform_edit(id.into(), value.into()) };
    }
}

// We can't fully construct the app_state until we have the ComponentHandler.
// So, store the whole state in its own struct
struct Inner<P: Params, E: Editor<P>> {
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<E>>,
    _phantom: PhantomData<P>
}

impl<P: Params, E: Editor<P>> Inner<P, E> {
    fn new(component_handler: VstPtr<dyn IComponentHandler>) -> Self {
        let host_handle = VST3HostHandle { component_handler };
		let app_state = Rc::new(RefCell::new(AppState::new(P::default(), host_handle)));
		let editor = Rc::new(RefCell::new(E::new()));
        Self {
            app_state,
            editor,
            _phantom: PhantomData
        }
    }

    fn get_parameter_info(&self, param_index: i32, info: &mut ParameterInfo) -> tresult {
		let app_state = RefCell::borrow(&self.app_state);
		let Some(param_ref) = app_state.parameters().get_by_index(param_index as usize) else { return kInvalidArgument };

		info.id = param_ref.id().into();
		info.flags = match param_ref {
			ParamRef::ByPass(_) => ParameterFlags::kCanAutomate as i32 | ParameterFlags::kIsBypass as i32,
			ParamRef::Int(_) => ParameterFlags::kCanAutomate as i32,
			ParamRef::Float(_) => ParameterFlags::kCanAutomate as i32,
			ParamRef::StringList(_) => ParameterFlags::kCanAutomate as i32 | ParameterFlags::kIsList as i32,
			ParamRef::Bool(_) => ParameterFlags::kCanAutomate as i32,
		};
		info.default_normalized_value = param_ref.default_normalized().into();
		strcpyw(param_ref.name(), &mut info.short_title);
		strcpyw(param_ref.name(), &mut info.title);
		info.step_count = param_ref.step_count() as i32;
		info.unit_id = kRootUnitId;
		strcpyw("unit", &mut info.units);
		kResultOk
    }

    fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
		let app_state = RefCell::borrow(&self.app_state);
		let Some(param_ref) = app_state.parameters().get_by_id(ParameterId::new(id)) else { return kInvalidArgument };
		param_ref.internal_set_value_normalized(unsafe { NormalizedValue::from_f64_unchecked(value) });
		kResultOk
    }

    fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
		let app_state = RefCell::borrow(&self.app_state);
		let value_normalized = unsafe { NormalizedValue::from_f64_unchecked(value_normalized) };
		app_state.parameters().get_by_id(ParameterId::new(id))
			.map_or(0.0, |param| param.denormalize(value_normalized).into())
    }

    fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
		let app_state = RefCell::borrow(&self.app_state);
		app_state.parameters().get_by_id(ParameterId::new(id))
        	.map_or(0.0, |param| param.normalize(PlainValue::new(plain_value)).into())
    }

    fn get_param_normalized(&self, id: u32) -> f64 {
		let app_state = RefCell::borrow(&self.app_state);
        app_state.parameters().get_by_id(ParameterId::new(id))
			.map_or(0.0, |p| p.get_normalized().into())
    }
}

#[VST3(implements(IEditController))]
pub struct EditController<P: Params, E: Editor<P>> {
    inner: OnceCell<Inner<P, E>>
}

impl<P: Params, E: Editor<P>> EditController<P, E> {
    pub const CID: IID = IID { data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 14] };

    pub fn new() -> Box<Self> {
        Self::allocate(OnceCell::new())
    }

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }
}

impl<P: Params, E: Editor<P>> IEditController for EditController<P, E> {
    unsafe fn set_component_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        // The actual write is done in IComponent
        kResultOk
    }

    unsafe fn get_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        // The actual read is done in IComponent
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        P::PARAMS.len() as i32
    }

    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
        self.inner.get()
            .map(|inner| {
                let info = unsafe { &mut *info };
                inner.get_parameter_info(param_index, info)
            })
            .unwrap_or(kResultFalse)
    }

    unsafe fn get_param_string_by_value(&self, _id: u32, _value_normalized: f64, _string: *mut tchar) -> tresult {
        kNotImplemented
    }

    unsafe fn get_param_value_by_string(&self, _id: u32, _string: *const tchar, _value_normalized: *mut f64) -> tresult {
        kNotImplemented
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        self.inner.get()
            .map(|inner| inner.normalized_param_to_plain(id, value_normalized))
            .unwrap_or(0.0)
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        self.inner.get()
            .map(|inner| inner.plain_param_to_normalized(id, plain_value))
            .unwrap_or(0.0)
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        self.inner.get()
            .map(|inner| inner.get_param_normalized(id))
            .unwrap_or(0.0)
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        self.inner.get()
            .map(|inner| inner.set_param_normalized(id, value))
            .unwrap_or(kResultFalse)
    }

    unsafe fn set_component_handler(&self, handler: SharedVstPtr<dyn IComponentHandler>) -> tresult {
        if let Some(handler) = handler.upgrade() {
            self.inner.set(Inner::new(handler)).map_or(kResultFalse, |_| kResultOk)
        } else {
            kInvalidArgument
        }
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        self.inner.get()
            .map(|inner| PlugView::create_instance(inner.app_state.clone(), inner.editor.clone()))
            .unwrap_or(std::ptr::null_mut())
    }
}

impl<P: Params, E: Editor<P>> IPluginBase for EditController<P, E> {
    unsafe fn initialize(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}