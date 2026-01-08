use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::param::{NormalizedValue, PlainValue};

use super::{
    AnyParameter, BoolParameter, ByPassParameter, FloatParameter, GroupId, IntParameter,
    ParamVisitor, ParameterId, StringListParameter, group::AnyParameterGroup,
    traversal::ParameterTraversal,
};

pub trait Params: ParameterTraversal {
    fn new() -> Self;
}

impl Params for () {
    fn new() -> Self {}
}

#[derive(Clone, Copy)]
enum ParamPtr {
    Float(*const FloatParameter),
    Int(*const IntParameter),
    StringList(*const StringListParameter),
    ByPass(*const ByPassParameter),
    Bool(*const BoolParameter),
}

#[derive(Clone, Copy)]
struct ParamEntry {
    ptr: ParamPtr,
    value_index: usize,
}

union ParamValue {
    float: f64,
    bool: bool,
    int: i64,
    string_list: usize,
}

pub enum ParamRef<'a> {
    Float((&'a FloatParameter, &'a Cell<f64>)),
    Int((&'a IntParameter, &'a Cell<i64>)),
    StringList((&'a StringListParameter, &'a Cell<usize>)),
    ByPass((&'a ByPassParameter, &'a Cell<bool>)),
    Bool((&'a BoolParameter, &'a Cell<bool>)),
}

impl<'a> ParamRef<'a> {
    fn new(p: ParamPtr, v: ParamValue) -> Self {
        match p {
            ParamPtr::Float(p) => todo!(),
            ParamPtr::Int(_) => todo!(),
            ParamPtr::StringList(_) => todo!(),
            ParamPtr::ByPass(_) => todo!(),
            ParamPtr::Bool(_) => todo!(),
        }
    }

    pub fn info(&self) -> &'a dyn AnyParameter {
        match self {
            Self::ByPass((p, _)) => *p,
            Self::Float((p, _)) => *p,
            Self::Int((p, _)) => *p,
            Self::StringList((p, _)) => *p,
            Self::Bool((p, _)) => *p,
        }
    }

    pub fn name(&self) -> &str {
        self.info().name()
    }

    pub fn id(&self) -> ParameterId {
        self.info().id()
    }

    pub fn default_value(&self) -> PlainValue {
        self.info().default_value_plain()
    }

    pub fn default_normalized(&self) -> NormalizedValue {
        self.normalize(self.default_value())
    }

    pub(crate) fn internal_set_value_normalized(&self, value: NormalizedValue) {
        match self {
            Self::Float(p) => p.set_value_normalized(value),
            Self::Int(p) => p.set_value_normalized(value),
            Self::StringList(p) => p.set_value_normalized(value),
            Self::ByPass(p) => p.set_value_normalized(value),
            Self::Bool(p) => p.set_value_normalized(value),
        }
    }

    pub(crate) fn internal_set_value_plain(&self, value: PlainValue) {
        self.internal_set_value_normalized(self.normalize(value))
    }

    pub fn plain_value(&self) -> PlainValue {
        match self {
            Self::Float(p) => p.plain_value(),
            Self::Int(p) => p.plain_value(),
            Self::StringList(p) => p.plain_value(),
            Self::ByPass(p) => p.plain_value(),
            Self::Bool(p) => p.plain_value(),
        }
    }

    pub fn normalized_value(&self) -> NormalizedValue {
        match self {
            Self::Float(p) => p.normalized_value(),
            Self::Int(p) => p.normalized_value(),
            Self::StringList(p) => p.normalized_value(),
            Self::ByPass(p) => p.normalized_value(),
            Self::Bool(p) => p.normalized_value(),
        }
    }

    pub fn value_as<T: Any>(&self) -> Option<T> {
        if TypeId::of::<T>() == TypeId::of::<PlainValue>() {
            Some(unsafe { std::mem::transmute(self.plain_value()) })
        } else if TypeId::of::<T>() == TypeId::of::<NormalizedValue>() {
            Some(unsafe { std::mem::transmute(self.normalized_value()) })
        } else {
            match self {
                Self::Float(p) if TypeId::of::<T>() == TypeId::of::<f64>() => {
                    Some(unsafe { std::mem::transmute(p.value()) })
                }
                Self::Int(p) if TypeId::of::<T>() == TypeId::of::<f64>() => {
                    Some(unsafe { std::mem::transmute(p.value()) })
                }
                Self::StringList(p) if TypeId::of::<T>() == TypeId::of::<i64>() => {
                    Some(unsafe { std::mem::transmute(p.value()) })
                }
                Self::ByPass(p) if TypeId::of::<T>() == TypeId::of::<bool>() => {
                    Some(unsafe { std::mem::transmute(p.value()) })
                }
                Self::Bool(p) if TypeId::of::<T>() == TypeId::of::<bool>() => {
                    Some(unsafe { std::mem::transmute(p.value()) })
                }
                _ => None,
            }
        }
    }

    pub fn normalize(&self, value: PlainValue) -> NormalizedValue {
        self.info().normalize(value)
    }

    pub fn denormalize(&self, value: NormalizedValue) -> PlainValue {
        self.info().denormalize(value)
    }

    pub fn step_count(&self) -> usize {
        self.info().step_count()
    }
}

pub struct ParamIter<'a> {
    inner_iter: std::slice::Iter<'a, (Option<GroupId>, ParamEntry)>,
}

impl<'a> Iterator for ParamIter<'a> {
    type Item = ParamRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next()
            .map(|&(_, p)| unsafe { &*p.ptr }.as_param_ref())
    }
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
    fn get_by_id(&self, id: ParameterId) -> Option<&dyn AnyParameter>;
    fn get_by_index(&self, index: usize) -> Option<(Option<GroupId>, &dyn AnyParameter)>;
    fn count(&self) -> usize;
    fn parameter_ids(&self) -> Vec<ParameterId>;
    fn get_group_by_index(&self, index: usize)
    -> Option<(Option<GroupId>, &dyn AnyParameterGroup)>;
    fn groups_count(&self) -> usize;
}

struct GatherParamPtrsVisitor<'a> {
    current_group_id: Option<GroupId>,
    bool_values: &'a mut Vec<bool>,
    float_values: &'a mut Vec<f64>,
    int_values: &'a mut Vec<i64>,
    enum_values: &'a mut Vec<usize>,
    params_vec: &'a mut Vec<(Option<GroupId>, ParamEntry)>,
    params_map: &'a mut HashMap<ParameterId, ParamEntry>,
    groups_vec: &'a mut Vec<(Option<GroupId>, *const dyn AnyParameterGroup)>,
}

impl GatherParamPtrsVisitor<'_> {
    fn add_param_ptr(&mut self, id: ParameterId, ptr: ParamPtr, v: ParamValue) {
        let value_index = self.values.len();
        let entry = ParamEntry { value_index, ptr };
        self.values.push(v);
        self.params_vec.push((self.current_group_id, entry));
        self.params_map.insert(id, entry);
    }
}

impl ParamVisitor for GatherParamPtrsVisitor<'_> {
    fn bool_parameter(&mut self, p: &super::BoolParameter) {
        let value_index = self.bool_values.len();
        self.bool_values.push(p.default_value());
        self.add_param_ptr(
            p.id(),
            ParamPtr::Bool(p as *const _),
            ParamValue { bool: false },
        );
    }

    fn bypass_parameter(&mut self, p: &super::ByPassParameter) {
        self.add_param_ptr(p);
    }

    fn float_parameter(&mut self, p: &super::FloatParameter) {
        self.add_param_ptr(p);
    }

    fn int_parameter(&mut self, p: &super::IntParameter) {
        self.add_param_ptr(p);
    }

    fn string_list_parameter(&mut self, p: &super::StringListParameter) {
        self.add_param_ptr(p);
    }

    fn group<P: ParameterTraversal>(&mut self, group: &super::ParameterGroup<P>) {
        self.groups_vec
            .push((self.current_group_id, group as *const _));

        let old_group_id = self.current_group_id;
        self.current_group_id = Some(group.id());
        group.children().visit(self);
        self.current_group_id = old_group_id;
    }
}

pub struct ParameterMap<P: Params> {
    parameters: P,
    bool_values: Vec<bool>,
    float_values: Vec<f64>,
    int_values: Vec<i64>,
    enum_values: Vec<usize>,
    params_vec: Vec<(Option<GroupId>, ParamEntry)>,
    params_map: HashMap<ParameterId, ParamEntry>,
    groups_vec: Vec<(Option<GroupId>, *const dyn AnyParameterGroup)>,
}

impl<P: Params> ParameterMap<P> {
    pub fn new(parameters: P) -> Rc<Self> {
        // Construct the instance first, so that parameters is moved into the correct memory location
        let mut this = Rc::new(Self {
            parameters,
            bool_values: Vec::new(),
            float_values: Vec::new(),
            int_values: Vec::new(),
            enum_values: Vec::new(),
            params_vec: Vec::new(),
            params_map: HashMap::new(),
            groups_vec: Vec::new(),
        });

        let this_ref = Rc::get_mut(&mut this).unwrap();
        let mut visitor = GatherParamPtrsVisitor {
            current_group_id: None,
            bool_values: &mut this_ref.bool_values,
            float_values: &mut this_ref.float_values,
            int_values: &mut this_ref.int_values,
            enum_values: &mut this_ref.enum_values,
            params_vec: &mut this_ref.params_vec,
            params_map: &mut this_ref.params_map,
            groups_vec: &mut this_ref.groups_vec,
        };
        this_ref.parameters.visit(&mut visitor);

        this
    }

    pub fn parameters_ref(&self) -> &P {
        &self.parameters
    }

    pub fn iter(&self) -> ParamIter<'_> {
        ParamIter {
            inner_iter: self.params_vec.as_slice().iter(),
        }
    }
}

impl<P: Params> AnyParameterMap for ParameterMap<P> {
    fn get_by_id(&self, id: ParameterId) -> Option<&dyn AnyParameter> {
        self.params_map.get(&id).map(|&p| unsafe { &*p })
    }

    fn get_by_index(&self, index: usize) -> Option<(Option<GroupId>, &dyn AnyParameter)> {
        self.params_vec
            .get(index)
            .map(|&(group_id, p)| (group_id, unsafe { &*p }))
    }

    fn count(&self) -> usize {
        self.params_vec.len()
    }

    fn parameter_ids(&self) -> Vec<ParameterId> {
        self.iter().map(|param_ref| param_ref.id()).collect()
    }

    fn groups_count(&self) -> usize {
        self.groups_vec.len()
    }

    fn get_group_by_index(
        &self,
        index: usize,
    ) -> Option<(Option<GroupId>, &dyn AnyParameterGroup)> {
        self.groups_vec
            .get(index)
            .map(|&(group_id, g)| (group_id, unsafe { &*g }))
    }
}
