use std::{ptr::NonNull, rc::Rc};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_audio_toolbox::{
    AUParameter, AUParameterAddress, AUParameterNode, AUParameterTree, AUValue,
    AudioUnitParameterOptions, AudioUnitParameterUnit,
};
use objc2_foundation::{ns_string, NSArray, NSMutableArray, NSNumber, NSString};

use crate::param::{
    AnyParameter, AnyParameterGroup, AnyParameterMap, ParamVisitor, ParameterId, ParameterMap,
    Params, PlainValue,
};

struct CreateParametersVisitor {
    au_params: Retained<NSMutableArray<AUParameterNode>>,
}

impl CreateParametersVisitor {
    pub fn new() -> Self {
        Self {
            au_params: NSMutableArray::<AUParameterNode>::new(),
        }
    }
}

fn create_parameter(
    identifier: &NSString,
    name: &NSString,
    address: AUParameterAddress,
    min: AUValue,
    max: AUValue,
    unit: AudioUnitParameterUnit,
    unit_name: &NSString,
    flags: AudioUnitParameterOptions,
    value_strings: &NSArray<NSString>,
    dependent_parameters: &NSArray<NSNumber>,
) -> Retained<AUParameter> {
    unsafe {
        AUParameterTree::createParameterWithIdentifier_name_address_min_max_unit_unitName_flags_valueStrings_dependentParameters(
			identifier,
			name,
			address,
			min,
			max,
			unit,
			Some(unit_name),
			flags,
			Some(value_strings),
			Some(dependent_parameters))
    }
}

impl ParamVisitor for CreateParametersVisitor {
    fn bool_parameter(&mut self, p: &crate::param::BoolParameter) {
        let au_param = create_parameter(
            &NSString::from_str(p.info().name()),
            &NSString::from_str(p.info().name()),
            p.info().id().into(),
            Into::<f64>::into(p.info().min_value()) as _,
            Into::<f64>::into(p.info().max_value()) as _,
            AudioUnitParameterUnit::Boolean,
            ns_string!("-"),
            AudioUnitParameterOptions::Flag_IsReadable | AudioUnitParameterOptions::Flag_IsWritable,
            &NSArray::new(),
            &NSArray::new(),
        );
        self.au_params.addObject(&au_param);
    }

    fn bypass_parameter(&mut self, p: &crate::param::ByPassParameter) {
        let au_param = create_parameter(
            &NSString::from_str(p.info().name()),
            &NSString::from_str(p.info().name()),
            p.info().id().into(),
            Into::<f64>::into(p.info().min_value()) as _,
            Into::<f64>::into(p.info().max_value()) as _,
            AudioUnitParameterUnit::Boolean,
            ns_string!("-"),
            AudioUnitParameterOptions::Flag_IsReadable | AudioUnitParameterOptions::Flag_IsWritable,
            &NSArray::new(),
            &NSArray::new(),
        );
        self.au_params.addObject(&au_param);
    }

    fn float_parameter(&mut self, p: &crate::param::FloatParameter) {
        let au_param = create_parameter(
            &NSString::from_str(p.info().name()),
            &NSString::from_str(p.info().name()),
            p.info().id().into(),
            Into::<f64>::into(p.info().min_value()) as _,
            Into::<f64>::into(p.info().max_value()) as _,
            AudioUnitParameterUnit::CustomUnit,
            ns_string!("Unit"),
            AudioUnitParameterOptions::Flag_IsReadable | AudioUnitParameterOptions::Flag_IsWritable,
            &NSArray::new(),
            &NSArray::new(),
        );
        self.au_params.addObject(&au_param);
    }

    fn int_parameter(&mut self, p: &crate::param::IntParameter) {
        let au_param = create_parameter(
            &NSString::from_str(p.info().name()),
            &NSString::from_str(p.info().name()),
            p.info().id().into(),
            Into::<f64>::into(p.info().min_value()) as _,
            Into::<f64>::into(p.info().max_value()) as _,
            AudioUnitParameterUnit::CustomUnit,
            ns_string!("Unit"),
            AudioUnitParameterOptions::Flag_IsReadable | AudioUnitParameterOptions::Flag_IsWritable,
            &NSArray::new(),
            &NSArray::new(),
        );
        self.au_params.addObject(&au_param);
    }

    fn string_list_parameter(&mut self, p: &crate::param::StringListParameter) {
        todo!()
    }

    fn group<P: crate::param::ParameterTraversal>(
        &mut self,
        group: &crate::param::ParameterGroup<P>,
    ) {
        let mut child_visitor = Self::new();
        group.children().visit(&mut child_visitor);
        let au_group = unsafe {
            AUParameterTree::createGroupWithIdentifier_name_children(
                &NSString::from_str(group.name()),
                &NSString::from_str(group.name()),
                &child_visitor.au_params,
            )
        };
        self.au_params.addObject(&au_group);
    }
}

pub fn create_parameter_tree<P: Params>(
    parameters: Rc<ParameterMap<P>>,
) -> Retained<AUParameterTree> {
    let mut visitor = CreateParametersVisitor::new();
    parameters.parameters_ref().visit(&mut visitor);
    let tree = unsafe { AUParameterTree::createTreeWithChildren(&visitor.au_params) };

    let value_observer = {
        let parameters = parameters.clone();
        RcBlock::new(move |p: NonNull<AUParameter>, value: AUValue| {
            let p = unsafe { p.as_ref() };
            let id = ParameterId(unsafe { p.address() } as _);
            if let Some(param_ref) = parameters.get_by_id(id) {
                param_ref.set_value_plain(PlainValue::new(value as _));
            }
        })
    };
    unsafe { tree.setImplementorValueObserver(RcBlock::into_raw(value_observer)) };
    let value_provider = {
        let parameters = parameters.clone();
        RcBlock::new(move |p: NonNull<AUParameter>| -> AUValue {
            let p = unsafe { p.as_ref() };
            let id = ParameterId(unsafe { p.address() } as _);
            parameters.get_by_id(id).map_or(0.0, |param| {
                let value: f64 = param.plain_value().into();
                value as _
            })
        })
    };
    unsafe { tree.setImplementorValueProvider(RcBlock::into_raw(value_provider)) };

    tree
}
