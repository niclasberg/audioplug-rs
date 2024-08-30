use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use vst3_com::VstPtr;
use vst3_com::vst::{kRootUnitId, ParameterFlags};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::{IID, VST3, c_void};
use vst3_sys::vst::{IComponentHandler, IConnectionPoint, IEditController, IHostApplication, IMessage, ParameterInfo};

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
struct Inner<E: Editor> {
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<E>>
}

impl<E: Editor> Inner<E> {
    fn new(component_handler: VstPtr<dyn IComponentHandler>) -> Self {
        let host_handle = VST3HostHandle { component_handler };
		let app_state = Rc::new(RefCell::new(AppState::new(E::Parameters::default(), host_handle)));
		let editor = Rc::new(RefCell::new(E::new()));
        Self {
            app_state,
            editor
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

#[VST3(implements(IEditController, IConnectionPoint))]
pub struct EditController<E: Editor> {
    inner: RefCell<Option<Inner<E>>>,
    host_context: Cell<Option<VstPtr<dyn IUnknown>>>,
    peer_connection: Cell<Option<VstPtr<dyn IConnectionPoint>>>,
}

impl<E: Editor> EditController<E> {
    pub const CID: IID = IID { data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 14] };

    pub fn new() -> Box<Self> {
        Self::allocate(RefCell::new(None), Cell::new(None), Cell::new(None))
    }

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }

    fn try_with_inner<R>(&self, f: impl FnOnce(&Inner<E>) -> R) -> Option<R> {
        if let Ok(inner) = self.inner.try_borrow() {
            inner.as_ref().map(f)
        } else {
            None
        }
    }
}

impl<E: Editor> IEditController for EditController<E> {
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
        E::Parameters::PARAMS.len() as i32
    }

    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
        self.try_with_inner(|inner| inner.get_parameter_info(param_index, unsafe { &mut *info }))
            .unwrap_or(kResultFalse)
    }

    unsafe fn get_param_string_by_value(&self, _id: u32, _value_normalized: f64, _string: *mut tchar) -> tresult {
        kNotImplemented
    }

    unsafe fn get_param_value_by_string(&self, _id: u32, _string: *const tchar, _value_normalized: *mut f64) -> tresult {
        kNotImplemented
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        self.try_with_inner(|inner| inner.normalized_param_to_plain(id, value_normalized))
            .unwrap_or(0.0)
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        self.try_with_inner(|inner| inner.plain_param_to_normalized(id, plain_value))
            .unwrap_or(0.0)
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        self.try_with_inner(|inner| inner.get_param_normalized(id))
            .unwrap_or(0.0)
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        self.try_with_inner(|inner| inner.set_param_normalized(id, value))
            .unwrap_or(kResultFalse)
    }

    unsafe fn set_component_handler(&self, handler: SharedVstPtr<dyn IComponentHandler>) -> tresult {
        *self.inner.borrow_mut() = handler.upgrade().map(|handler| Inner::new(handler));
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        self.try_with_inner(|inner| PlugView::create_instance(inner.app_state.clone(), inner.editor.clone()))
            .unwrap_or(std::ptr::null_mut())
    }
}

impl<E: Editor> IPluginBase for EditController<E> {
    unsafe fn initialize(&self, context: *mut c_void) -> tresult {
        /*let old_host_context = self.host_context.take();
        if old_host_context.is_some() {
            self.host_context.set(old_host_context);
            return kResultFalse;
        }

        if let Some(context) = VstPtr::<dyn IUnknown>::owned(context as *mut _) {
            self.host_context.set(Some(context));
            kResultOk
        } else {
            kInvalidArgument
        }*/
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        /*self.inner.replace(None);
        self.host_context.replace(None);
        // Clear in case the host did not call disconnect
        self.peer_connection.take();*/
        kResultOk
    }
}

impl<E: Editor> IConnectionPoint for EditController<E> {
	unsafe fn connect(&self, other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        if let Some(other) = other.upgrade() {
            self.peer_connection.set(Some(other));
            //other.notify(message)
        }
		kResultOk
	}

	unsafe fn disconnect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        self.peer_connection.take();
		kResultOk
	}

	unsafe fn notify(&self, _message: SharedVstPtr<dyn IMessage>) -> tresult {
		kResultOk
	}
}