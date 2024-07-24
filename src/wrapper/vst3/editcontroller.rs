use std::cell::{RefCell, Cell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use vst3_com::VstPtr;
use vst3_com::vst::{kRootUnitId, ParameterFlags};
use vst3_sys::base::*;
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::{IID, VST3, c_void};
use vst3_sys::vst::{IEditController, ParameterInfo, IComponentHandler};

use vst3_sys as vst3_com;

use crate::app::AppState;
use crate::param::{NormalizedValue, ParamRef, ParameterGetter, ParameterId, Params, PlainValue};

use super::plugview::PlugView;
use super::util::strcpyw;


#[VST3(implements(IEditController))]
pub struct EditController<P: Params> {
    component_handler: Cell<Option<VstPtr<dyn IComponentHandler>>>,
    parameters: HashMap<ParameterId, ParameterGetter<P>>,
    app_state: Rc<RefCell<AppState>>,
    _phantom: PhantomData<P>
}

impl<P: Params> EditController<P> {
    pub const CID: IID = IID { data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 14] };

    pub fn new() -> Box<Self> {
        let params = P::default();
        let parameters: HashMap<ParameterId, ParameterGetter<P>> = P::PARAMS.iter()
			.map(|getter| (getter(&params).id(), *getter))
			.collect();
        Self::allocate(Cell::new(None), parameters, Rc::new(RefCell::new(AppState::new(params))), PhantomData)
    }

	fn with_param_ref<T>(&self, id: ParameterId, f: impl FnOnce(Option<ParamRef<'_>>) -> T) -> T {
		if let Some(getter) = self.parameters.get(&id) {
			let app_state = RefCell::borrow(&self.app_state);
			let params: &P = app_state.parameters_as().unwrap();
			f(Some(getter(&params)))
		} else {
			f(None)
		}
	}

    pub fn create_instance() -> *mut c_void {
        Box::into_raw(Self::new()) as *mut c_void
    }
}

impl<P: Params> IEditController for EditController<P> {
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
		if param_index < 0 || param_index >= P::PARAMS.len() as i32 {
			return kInvalidArgument;
		}

		let app_state = RefCell::borrow(&self.app_state);
		let params: &P = app_state.parameters_as().unwrap();
		let param_ref = (P::PARAMS[param_index as usize])(params);

		let info = &mut *info;

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

    unsafe fn get_param_string_by_value(&self, id: u32, value_normalized: f64, string: *mut tchar) -> tresult {
        kNotImplemented
    }

    unsafe fn get_param_value_by_string(&self, id: u32, string: *const tchar, value_normalized: *mut f64) -> tresult {
        kNotImplemented
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        self.with_param_ref(ParameterId::new(id), |param_ref| {
            let value_normalized = NormalizedValue::from_f64_unchecked(value_normalized);
            param_ref.map_or(0.0, |param| param.denormalize(value_normalized).into())
        })
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, plain_value: f64) -> f64 {
        self.with_param_ref(ParameterId::new(id), |param_ref| {
            param_ref.map_or(0.0, |param| param.normalize(PlainValue::new(plain_value)).into())
        })
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        self.with_param_ref(ParameterId::new(id), |param_ref| {
            param_ref.map_or(0.0, |p| p.get_normalized().into())
        })
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
		self.with_param_ref(ParameterId::new(id), |param_ref| {
			if let Some(param_ref) = param_ref {
				param_ref.internal_set_value_normalized(NormalizedValue::from_f64_unchecked(value));
				kResultOk
			} else {
				kInvalidArgument
			}
        })
    }

    unsafe fn set_component_handler(&self, handler: SharedVstPtr<dyn IComponentHandler>) -> tresult {
        self.component_handler.replace(handler.upgrade());
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        // Take, clone and put back. Maybe we should just use a RefCell instead?
        let component_handler = self.component_handler.take();
		self.component_handler.set(component_handler.clone());
		
		component_handler.map_or(std::ptr::null_mut(), |component_handler|
			PlugView::<P>::create_instance(component_handler, self.app_state.clone()))
        
    }
}

impl<P: Params> IPluginBase for EditController<P> {
    unsafe fn initialize(&self, _context: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}