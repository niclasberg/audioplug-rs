use std::cell::{Cell, RefCell};
use std::rc::Rc;

use vst3::Steinberg::Vst::ParameterInfo_::ParameterFlags_;
use vst3::Steinberg::Vst::{
    IComponentHandler, IComponentHandlerTrait, IConnectionPoint, IConnectionPointTrait,
    IEditController, IEditControllerTrait, IHostApplication, IMessage, IUnitInfo, IUnitInfoTrait,
    ParamValue, ParameterInfo, ProgramListInfo, String128, TChar, UnitInfo, kNoParentUnitId,
    kNoProgramListId, kRootUnitId,
};
use vst3::Steinberg::{
    FIDString, FUnknown, IBStream, IPlugView, IPluginBaseTrait, kInvalidArgument, kNotImplemented,
    kResultFalse, kResultOk, tresult,
};
use vst3::{ComPtr, ComRef, ComWrapper};

use crate::param::{
    AnyParameterMap, NormalizedValue, ParamRef, ParameterId, ParameterMap, Params, PlainValue,
};
use crate::ui::{AppState, HostHandle};
use crate::{Editor, EditorContext, platform};

use super::plugview::PlugView;
use super::util::strcpyw;

struct VST3HostHandle {
    component_handler: ComPtr<IComponentHandler>,
    is_editing_parameters_from_gui: Rc<Cell<bool>>,
}

impl HostHandle for VST3HostHandle {
    fn begin_edit(&self, id: ParameterId) {
        unsafe { self.component_handler.beginEdit(id.into()) };
    }

    fn end_edit(&self, id: ParameterId) {
        unsafe { self.component_handler.endEdit(id.into()) };
    }

    fn perform_edit(&self, info: &dyn crate::param::AnyParameter, value: NormalizedValue) {
        self.is_editing_parameters_from_gui.replace(true);
        unsafe {
            self.component_handler
                .performEdit(info.id().into(), value.into())
        };
        self.is_editing_parameters_from_gui.replace(false);
    }
}

pub struct EditController<E: Editor> {
    app_state: Rc<RefCell<AppState>>,
    editor: Rc<RefCell<E>>,
    host_context: Cell<Option<ComPtr<IHostApplication>>>,
    peer_connection: Cell<Option<ComPtr<IConnectionPoint>>>,
    executor: Rc<platform::Executor>,
    is_editing_parameters_from_gui: Rc<Cell<bool>>,
    parameters: Rc<ParameterMap<E::Parameters>>,
}

impl<E: Editor> vst3::Class for EditController<E> {
    type Interfaces = (IEditController, IConnectionPoint, IUnitInfo);
}

impl<E: Editor> EditController<E> {
    pub fn new() -> Self {
        let executor = Rc::new(platform::Executor::new().unwrap());
        let parameters = ParameterMap::new(E::Parameters::new());
        let mut app_state = AppState::new(parameters.clone());
        let editor = Rc::new(RefCell::new(E::new(&mut EditorContext {
            app_state: &mut app_state,
        })));
        let app_state = Rc::new(RefCell::new(app_state));
        let is_editing_parameters_from_gui = Rc::new(Cell::new(false));
        Self {
            app_state,
            editor,
            host_context: Cell::new(None),
            peer_connection: Cell::new(None),
            executor,
            is_editing_parameters_from_gui,
            parameters,
        }
    }
}

#[allow(non_snake_case)]
impl<E: Editor> IEditControllerTrait for EditController<E> {
    unsafe fn setComponentState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getParameterCount(&self) -> i32 {
        self.parameters.count() as i32
    }

    unsafe fn getParameterInfo(&self, param_index: i32, info: *mut ParameterInfo) -> tresult {
        let Some((group_id, param_ref)) = self.parameters.get_by_index(param_index as usize) else {
            return kInvalidArgument;
        };
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        let parameter_id = param_ref.id();

        info.id = parameter_id.into();
        info.flags = match param_ref {
            ParamRef::ByPass(_) => ParameterFlags_::kCanAutomate | ParameterFlags_::kIsBypass,
            ParamRef::Int(_) => ParameterFlags_::kCanAutomate,
            ParamRef::Float(_) => ParameterFlags_::kCanAutomate,
            ParamRef::StringList(_) => ParameterFlags_::kCanAutomate | ParameterFlags_::kIsList,
            ParamRef::Bool(_) => ParameterFlags_::kCanAutomate,
        };
        info.defaultNormalizedValue = param_ref.info().default_value_normalized().into();
        strcpyw(param_ref.name(), &mut info.shortTitle);
        strcpyw(param_ref.name(), &mut info.title);
        info.stepCount = param_ref.info().step_count() as i32;
        info.unitId = group_id.map(i32::from).unwrap_or(kRootUnitId);
        strcpyw("unit", &mut info.units);
        kResultOk
    }

    unsafe fn getParamStringByValue(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut String128,
    ) -> tresult {
        let Some(string) = (unsafe { string.as_mut() }) else {
            return kInvalidArgument;
        };
        let Some(param_ref) = self.parameters.get_by_id(ParameterId(id)) else {
            return kInvalidArgument;
        };
        let Some(value) = NormalizedValue::from_f64(value_normalized) else {
            return kInvalidArgument;
        };
        let value_str = param_ref.info().string_from_value(value);

        strcpyw(&value_str, string);
        kResultOk
    }

    unsafe fn getParamValueByString(
        &self,
        id: u32,
        string: *mut TChar,
        valueNormalized: *mut ParamValue,
    ) -> tresult {
        let Some(param_ref) = self.parameters.get_by_id(ParameterId(id)) else {
            return kInvalidArgument;
        };
        let Some(value_normalized) = (unsafe { valueNormalized.as_mut() }) else {
            return kInvalidArgument;
        };

        let len = (0..)
            .take_while(|&i| unsafe { *string.offset(i) } != 0)
            .count();
        let slice = unsafe { std::slice::from_raw_parts(string as *mut _, len) };

        let Ok(str) = String::from_utf16(slice) else {
            return kInvalidArgument;
        };
        let Ok(value) = param_ref.info().value_from_string(&str) else {
            return kInvalidArgument;
        };
        *value_normalized = value.0;

        kResultOk
    }

    unsafe fn normalizedParamToPlain(&self, id: u32, value_normalized: f64) -> f64 {
        let value_normalized = NormalizedValue::from_f64_unchecked(value_normalized);
        self.parameters
            .get_by_id(ParameterId(id))
            .map_or(0.0, |param| {
                param.info().denormalize(value_normalized).into()
            })
    }

    unsafe fn plainParamToNormalized(&self, id: u32, plain_value: f64) -> f64 {
        self.parameters
            .get_by_id(ParameterId(id))
            .map_or(0.0, |param| {
                param.info().normalize(PlainValue::new(plain_value)).into()
            })
    }

    unsafe fn getParamNormalized(&self, id: u32) -> f64 {
        self.parameters
            .get_by_id(ParameterId(id))
            .map_or(0.0, |p| p.normalized_value().into())
    }

    unsafe fn setParamNormalized(&self, id: u32, value: f64) -> tresult {
        // Avoid re-entrancy issues when setting a parameter from the ui
        if self.is_editing_parameters_from_gui.get() {
            return kResultOk;
        }

        let id = ParameterId(id);
        let Some(value) = NormalizedValue::from_f64(value) else {
            return kInvalidArgument;
        };
        let mut app_state = self.app_state.borrow_mut();
        if app_state.set_normalized_parameter_value_from_host(id, value) {
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
        if let Some(component_handler) = unsafe { ComRef::from_raw(handler) } {
            let is_editing_parameters = self.is_editing_parameters_from_gui.clone();
            let handle = Box::new(VST3HostHandle {
                component_handler: component_handler.to_com_ptr(),
                is_editing_parameters_from_gui: is_editing_parameters,
            });
            self.app_state.borrow_mut().set_host_handle(Some(handle));
        } else {
            self.app_state.borrow_mut().set_host_handle(None);
        }

        kResultOk
    }

    unsafe fn createView(&self, _name: FIDString) -> *mut IPlugView {
        ComWrapper::new(PlugView::new(
            self.app_state.clone(),
            self.editor.clone(),
            self.parameters.clone(),
        ))
        .to_com_ptr()
        .expect("We are casting from a ComWrapper<IPlugView> to a ComPtr<IPlugView>, this should never fail")
        .into_raw()
    }
}

impl<E: Editor> IPluginBaseTrait for EditController<E> {
    unsafe fn initialize(&self, context: *mut FUnknown) -> tresult {
        let old_host_context = self.host_context.take();
        if old_host_context.is_some() {
            self.host_context.set(old_host_context);
            return kResultFalse;
        }

        let host_context =
            unsafe { ComRef::from_raw(context) }.and_then(|cx| cx.cast::<IHostApplication>());
        if let Some(context) = host_context {
            self.host_context.set(Some(context));
            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn terminate(&self) -> tresult {
        self.host_context.replace(None);
        // Clear in case the host did not call disconnect
        //self.peer_connection.take();
        kResultOk
    }
}

impl<E: Editor> IConnectionPointTrait for EditController<E> {
    unsafe fn connect(&self, _other: *mut IConnectionPoint) -> tresult {
        //if let Some(other) = other.upgrade() {
        //self.peer_connection.set(Some(other));
        //other.notify(message)
        //}
        kResultOk
    }

    unsafe fn disconnect(&self, _other: *mut IConnectionPoint) -> tresult {
        //self.peer_connection.take();
        kResultOk
    }

    unsafe fn notify(&self, _message: *mut IMessage) -> tresult {
        kResultOk
    }
}

#[allow(non_snake_case)]
impl<E: Editor> IUnitInfoTrait for EditController<E> {
    unsafe fn getUnitCount(&self) -> i32 {
        1 + self.parameters.groups_count() as i32
    }

    unsafe fn getUnitInfo(&self, unit_index: i32, info: *mut UnitInfo) -> tresult {
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };

        if unit_index == 0 {
            info.id = kRootUnitId;
            info.parentUnitId = kNoParentUnitId;
            info.programListId = kNoProgramListId;
            strcpyw("Root unit", &mut info.name);

            kResultOk
        } else {
            let Some((parent_group_id, group)) = self
                .parameters
                .get_group_by_index((unit_index as usize) - 1)
            else {
                return kInvalidArgument;
            };
            info.id = group.id().0 as _;
            info.parentUnitId = parent_group_id.map(|id| id.0 as i32).unwrap_or(kRootUnitId);
            info.programListId = kNoProgramListId;
            strcpyw(group.name(), &mut info.name);

            kResultOk
        }
    }

    unsafe fn getProgramListCount(&self) -> i32 {
        0
    }

    unsafe fn getProgramListInfo(&self, _list_index: i32, _info: *mut ProgramListInfo) -> tresult {
        kNotImplemented
    }

    unsafe fn getProgramName(
        &self,
        _list_id: i32,
        _program_index: i32,
        _name: *mut String128,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn getProgramInfo(
        &self,
        _list_id: i32,
        _program_index: i32,
        _attribute_id: *const i8,
        _attribute_value: *mut String128,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn hasProgramPitchNames(&self, _id: i32, _index: i32) -> tresult {
        kResultFalse
    }

    unsafe fn getProgramPitchName(
        &self,
        _id: i32,
        _index: i32,
        _pitch: i16,
        _name: *mut String128,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn getSelectedUnit(&self) -> i32 {
        0
    }

    unsafe fn selectUnit(&self, _id: i32) -> tresult {
        kResultOk
    }

    unsafe fn getUnitByBus(
        &self,
        _type_: i32,
        _dir: i32,
        _bus_index: i32,
        _channel: i32,
        _unit_id: *mut i32,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn setUnitProgramData(
        &self,
        _list_or_unit: i32,
        _program_idx: i32,
        _data: *mut IBStream,
    ) -> tresult {
        kNotImplemented
    }
}
