use std::cell::{Cell, RefCell};
use std::rc::Rc;

use vst3_com::VstPtr;
use vst3_com::vst::{kRootUnitId, ParameterFlags};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::{VST3, c_void};
use vst3_sys::vst::{IComponentHandler, IConnectionPoint, IEditController, IMessage, IUnitInfo, ParameterInfo, ProgramListInfo, String128, UnitInfo};

use vst3_sys as vst3_com;

use crate::app::{AppState, HostHandle};
use crate::param::{AnyParameterMap, NormalizedValue, ParamRef, ParameterId, ParameterMap, Params, PlainValue};
use crate::{platform, Editor};

use super::plugview::PlugView;
use super::util::strcpyw;

struct VST3HostHandle {
    component_handler: VstPtr<dyn IComponentHandler>,
	is_editing_parameters_from_gui: Rc<Cell<bool>>
}

impl HostHandle for VST3HostHandle {
    fn begin_edit(&self, id: ParameterId) {
        unsafe { self.component_handler.begin_edit(id.into()) };
    }

    fn end_edit(&self, id: ParameterId) {
        unsafe { self.component_handler.end_edit(id.into()) };
    }

    fn perform_edit(&self, info: &dyn crate::param::ParameterInfo, value: NormalizedValue) {
		self.is_editing_parameters_from_gui.replace(true);
        unsafe { self.component_handler.perform_edit(info.id().into(), value.into()) };
		self.is_editing_parameters_from_gui.replace(false);
    }
}

#[VST3(implements(IEditController, IConnectionPoint, IUnitInfo))]
pub struct EditController<E: Editor> {
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<E>>,
    host_context: Cell<Option<VstPtr<dyn IUnknown>>>,
    peer_connection: Cell<Option<VstPtr<dyn IConnectionPoint>>>,
    executor: Rc<platform::Executor>,
	is_editing_parameters_from_gui: Rc<Cell<bool>>,
    parameters: Rc<ParameterMap<E::Parameters>>
}

impl<E: Editor> EditController<E> {
    pub fn new() -> Box<Self> {
        let executor = Rc::new(platform::Executor::new().unwrap());
        let parameters = Rc::new(ParameterMap::new(E::Parameters::new()));
		let app_state = Rc::new(RefCell::new(AppState::new(parameters.clone(), executor.clone())));
		let editor = Rc::new(RefCell::new(E::new()));
		let is_editing_parameters = Rc::new(Cell::new(false));
        Self::allocate(app_state, editor, Cell::new(None), Cell::new(None), executor, is_editing_parameters, parameters)
    }

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }
}

impl<E: Editor> IEditController for EditController<E> {
    unsafe fn set_component_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn set_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: SharedVstPtr<dyn IBStream>) -> tresult {
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        self.parameters.count() as i32
    }

    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
		let Some(param_ref) = self.parameters.get_by_index(param_index as usize) else { return kInvalidArgument };
		let param_ref = param_ref.as_param_ref();

        let info = &mut *info;
		let parameter_id = param_ref.id();

		info.id = parameter_id.into();
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
		info.unit_id = self.parameters.get_group_id(parameter_id).map(i32::from).unwrap_or(kRootUnitId);
		strcpyw("unit", &mut info.units);
		kResultOk
    }

    unsafe fn get_param_string_by_value(&self, id: u32, value_normalized: f64, string: *mut tchar) -> tresult {
        // The string is actually a String128, it's mistyped in vst3-sys
        let string = string as *mut String128;
		let Some(param_ref) = self.parameters.get_by_id(ParameterId::new(id)) else { return kInvalidArgument };
        let Some(value) = NormalizedValue::from_f64(value_normalized) else { return kInvalidArgument };
        let value_str = param_ref.info().string_from_value(value);

        strcpyw(&value_str, &mut *string);
        kResultOk
    }

    unsafe fn get_param_value_by_string(&self, id: u32, string: *const tchar, value_normalized: *mut f64) -> tresult {
		let Some(param_ref) = self.parameters.get_by_id(ParameterId::new(id)) else { return kInvalidArgument };
        
        let len = (0..).take_while(|&i| unsafe { *string.offset(i) } != 0).count();
        let slice = unsafe { std::slice::from_raw_parts(string as *mut u16, len) };

        let Ok(str) = String::from_utf16(slice) else { return kInvalidArgument };
        let Ok(value) = param_ref.info().value_from_string(&str) else { return kInvalidArgument };
        *value_normalized = value.0;

        kResultOk
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
		let value_normalized = unsafe { NormalizedValue::from_f64_unchecked(value_normalized) };
		self.parameters.get_by_id(ParameterId::new(id))
			.map_or(0.0, |param| param.info().denormalize(value_normalized).into())
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
		self.parameters.get_by_id(ParameterId::new(id))
        	.map_or(0.0, |param| param.info().normalize(PlainValue::new(plain_value)).into())
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        self.parameters.get_by_id(ParameterId::new(id))
			.map_or(0.0, |p| p.normalized_value().into())
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
		// Avoid re-entrancy issues when setting a parameter from the ui
		if self.is_editing_parameters_from_gui.get() {
			return kResultOk;
		}

		let id = ParameterId::new(id);
		let Some(value) = NormalizedValue::from_f64(value) else { return kInvalidArgument };
		let mut app_state = self.app_state.borrow_mut();
		if app_state.set_normalized_parameter_value_from_host(id, value) {
			kResultOk
		} else {
			kInvalidArgument
		}
    }

    unsafe fn set_component_handler(&self, handler: SharedVstPtr<dyn IComponentHandler>) -> tresult {
        if let Some(component_handler) = handler.upgrade() {
			let is_editing_parameters = self.is_editing_parameters_from_gui.clone();
            let handle = Box::new(VST3HostHandle { component_handler, is_editing_parameters_from_gui: is_editing_parameters });
            self.app_state.borrow_mut().set_host_handle(Some(handle));
        } else {
            self.app_state.borrow_mut().set_host_handle(None);
        }
        
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        PlugView::create_instance(self.app_state.clone(), self.editor.clone(), self.parameters.clone())
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
        //self.host_context.replace(None);
        // Clear in case the host did not call disconnect
        //self.peer_connection.take();
        kResultOk
    }
}

impl<E: Editor> IConnectionPoint for EditController<E> {
	unsafe fn connect(&self, other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        if let Some(other) = other.upgrade() {
            //self.peer_connection.set(Some(other));
            //other.notify(message)
        }
		kResultOk
	}

	unsafe fn disconnect(&self, _other: SharedVstPtr<dyn IConnectionPoint>) -> tresult {
        //self.peer_connection.take();
		kResultOk
	}

	unsafe fn notify(&self, _message: SharedVstPtr<dyn IMessage>) -> tresult {
		kResultOk
	}
}

impl<E: Editor> IUnitInfo for EditController<E> {
	unsafe fn get_unit_count(&self) -> i32 {
		self.parameters.groups_count() as _
	}

	unsafe fn get_unit_info(&self, unit_index: i32, info: *mut UnitInfo) -> tresult {
        let Some(group) = self.parameters.get_group_by_index(unit_index as _) else { return kInvalidArgument };
		
        if info.is_null() {
            return kInvalidArgument;
        }

        let mut info = &mut *info;
        info.

        kResultOk
	}

	unsafe fn get_program_list_count(&self) -> i32 {
		0
	}

	unsafe fn get_program_list_info(&self, list_index: i32, info: *mut ProgramListInfo) -> tresult {
		kNotImplemented
	}

	unsafe fn get_program_name(&self, list_id: i32, program_index: i32, name: *mut u16) -> tresult {
		kNotImplemented
	}

	unsafe fn get_program_info(&self, list_id: i32, program_index: i32, attribute_id: *const u8, attribute_value: *mut u16) -> tresult {
		kNotImplemented
	}

	unsafe fn has_program_pitch_names(&self, id: i32, index: i32) -> tresult {
		kResultFalse
	}

	unsafe fn get_program_pitch_name(&self, id: i32, index: i32, pitch: i16, name: *mut u16,) -> tresult {
		kNotImplemented
	}

	unsafe fn get_selected_unit(&self) -> i32 {
		0
	}

	unsafe fn select_unit(&self,id:i32) -> tresult {
		kResultFalse
	}

	unsafe fn get_unit_by_bus(&self,type_:i32,dir:i32,bus_index:i32,channel:i32,unit_id: *mut i32,) -> tresult {
		kNotImplemented
	}

	unsafe fn set_unit_program_data(&self,list_or_unit:i32,program_idx:i32,data:SharedVstPtr<dyn IBStream> ,) -> tresult {
		kNotImplemented
	}
}