use std::cell::RefCell;
use std::rc::Rc;

use vst3_com::VstPtr;
use vst3_com::vst::{kRootUnitId, ParameterFlags};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::{IID, VST3, c_void};
use vst3_sys::vst::{IEditController, ParameterInfo, IComponentHandler};

use vst3_sys as vst3_com;

use crate::app::AppState;
use crate::param::{Parameter, FloatParameter, FloatRange, NormalizedValue, PlainValue};

use super::plugview::PlugView;
use super::util::strcpyw;


#[VST3(implements(IEditController))]
pub struct EditController {
    component_handler: RefCell<Option<VstPtr<dyn IComponentHandler>>>,
    parameters: Vec<Parameter>,
    parameter_values: RefCell<Vec<NormalizedValue>>,
    app_state: Rc<RefCell<AppState>>
}

impl EditController {
    pub const CID: IID = IID { data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 14] };

    pub fn new() -> Box<Self> {
        let parameters: Vec<Parameter> = vec![
            FloatParameter::new("param")
                .with_range(FloatRange::Linear { min: 0.0, max: 2.0 }).into(),
            FloatParameter::new("test").into()
        ];
        let parameter_values: Vec<NormalizedValue> = parameters.iter().map(|param| {
            param.default_normalized()
        }).collect();

        Self::allocate(RefCell::new(None), parameters, RefCell::new(parameter_values), Rc::new(RefCell::new(AppState::new())))
    }

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }
}

impl IEditController for EditController {
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
        self.parameters.len() as i32
    }

    unsafe fn get_parameter_info(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
        if let Some(parameter) = self.parameters.get(param_index as usize) {
            let info = &mut *info;

            info.id = param_index as u32;
            info.flags = match parameter {
                Parameter::ByPass => ParameterFlags::kCanAutomate as i32 | ParameterFlags::kIsBypass as i32,
                Parameter::Int(_) => ParameterFlags::kCanAutomate as i32,
                Parameter::Float(_) => ParameterFlags::kCanAutomate as i32,
                Parameter::StringList(_) => ParameterFlags::kCanAutomate as i32 | ParameterFlags::kIsList as i32,
                Parameter::Bool(_) => ParameterFlags::kCanAutomate as i32,
            };
            info.default_normalized_value = parameter.default_normalized().into();
            strcpyw(parameter.name(), &mut info.short_title);
            strcpyw(parameter.name(), &mut info.title);
            info.step_count = parameter.step_count() as i32;
            info.unit_id = kRootUnitId;
            strcpyw("unit", &mut info.units);
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn get_param_string_by_value(&self, id: u32, value_normalized: f64, string: *mut tchar) -> tresult {
        kNotImplemented
    }

    unsafe fn get_param_value_by_string(&self, id: u32, string: *const tchar, value_normalized: *mut f64) -> tresult {
        kNotImplemented
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        let value_normalized = NormalizedValue::from_f64_unchecked(value_normalized);
        self.parameters
            .get(id as usize)
            .map_or(0.0, |param| param.denormalize(value_normalized).into())
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        let plain_value = PlainValue::new(plain_value);
        self.parameters
            .get(id as usize)
            .map_or(0.0, |param| param.normalize(plain_value).into())
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        let values = self.parameter_values.borrow();
		self.parameters
            .get(id as usize)
            .map_or(0.0, |param| param.denormalize(*values.get_unchecked(id as usize)).into())
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
		let mut parameter_values = self.parameter_values.borrow_mut();
		if let Some(value_ref) = parameter_values.get_mut(id as usize) {
			*value_ref = NormalizedValue::from_f64_unchecked(value);
			kResultOk
		} else {
			kInvalidArgument
		}
    }

    unsafe fn set_component_handler(&self, handler: SharedVstPtr<dyn IComponentHandler>) -> tresult {
        *self.component_handler.borrow_mut() = handler.upgrade();
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        PlugView::create_instance((*self.component_handler.borrow()).clone(), self.app_state.clone())
    }
}

impl IPluginBase for EditController {
    unsafe fn initialize(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}