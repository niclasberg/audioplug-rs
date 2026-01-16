use std::{ptr::NonNull, rc::Rc};

use rustc_hash::FxHashMap;

use crate::param::{NormalizedValue, Parameter, PlainValue};

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
    Float(NonNull<FloatParameter>),
    Int(NonNull<IntParameter>),
    StringList(NonNull<StringListParameter>),
    ByPass(NonNull<ByPassParameter>),
    Bool(NonNull<BoolParameter>),
}

pub enum ParamRef<'a> {
    Float(&'a FloatParameter),
    Int(&'a IntParameter),
    StringList(&'a StringListParameter),
    ByPass(&'a ByPassParameter),
    Bool(&'a BoolParameter),
}

impl<'a> ParamRef<'a> {
    unsafe fn from_param_ptr(p: ParamPtr) -> Self {
        match p {
            ParamPtr::Float(p) => unsafe { ParamRef::Float(p.as_ref()) },
            ParamPtr::Int(p) => unsafe { ParamRef::Int(p.as_ref()) },
            ParamPtr::StringList(p) => unsafe { ParamRef::StringList(p.as_ref()) },
            ParamPtr::ByPass(p) => unsafe { ParamRef::ByPass(p.as_ref()) },
            ParamPtr::Bool(p) => unsafe { ParamRef::Bool(p.as_ref()) },
        }
    }

    pub fn info(&self) -> &'a dyn AnyParameter {
        match self {
            Self::ByPass(p) => *p,
            Self::Float(p) => *p,
            Self::Int(p) => *p,
            Self::StringList(p) => *p,
            Self::Bool(p) => *p,
        }
    }

    pub fn name(&self) -> &str {
        self.info().name()
    }

    pub fn id(&self) -> ParameterId {
        self.info().id()
    }

    pub(crate) fn set_value_normalized(&self, value: NormalizedValue) {
        match self {
            Self::Float(p) => p.set_value(p.value_from_normalized(value)),
            Self::Int(p) => p.set_value(p.value_from_normalized(value)),
            Self::StringList(p) => p.set_value(p.value_from_normalized(value)),
            Self::ByPass(p) => p.set_value(p.value_from_normalized(value)),
            Self::Bool(p) => p.set_value(p.value_from_normalized(value)),
        }
    }

    pub(crate) fn set_value_plain(&self, value: PlainValue) {
        match self {
            Self::Float(p) => p.set_value(p.value_from_plain(value)),
            Self::Int(p) => p.set_value(p.value_from_plain(value)),
            Self::StringList(p) => p.set_value(p.value_from_plain(value)),
            Self::ByPass(p) => p.set_value(p.value_from_plain(value)),
            Self::Bool(p) => p.set_value(p.value_from_plain(value)),
        }
    }

    pub fn plain_value(&self) -> PlainValue {
        match self {
            Self::Float(p) => p.plain_value(p.value()),
            Self::Int(p) => p.plain_value(p.value()),
            Self::StringList(p) => p.plain_value(p.value()),
            Self::ByPass(p) => p.plain_value(p.value()),
            Self::Bool(p) => p.plain_value(p.value()),
        }
    }

    pub fn normalized_value(&self) -> NormalizedValue {
        match self {
            Self::Float(p) => p.normalized_value(p.value()),
            Self::Int(p) => p.normalized_value(p.value()),
            Self::StringList(p) => p.normalized_value(p.value()),
            Self::ByPass(p) => p.normalized_value(p.value()),
            Self::Bool(p) => p.normalized_value(p.value()),
        }
    }
}

pub struct ParamIter<'a> {
    inner_iter: std::slice::Iter<'a, (Option<GroupId>, ParamPtr)>,
}

impl<'a> Iterator for ParamIter<'a> {
    type Item = ParamRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next()
            .map(|(_, p)| unsafe { ParamRef::from_param_ptr(*p) })
    }
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
    fn get_by_id<'s>(&'s self, id: ParameterId) -> Option<ParamRef<'s>>;
    fn get_by_index<'s>(&'s self, index: usize) -> Option<(Option<GroupId>, ParamRef<'s>)>;
    fn count(&self) -> usize;
    fn parameter_ids(&self) -> Vec<ParameterId>;
    fn get_group_by_index(&self, index: usize)
    -> Option<(Option<GroupId>, &dyn AnyParameterGroup)>;
    fn groups_count(&self) -> usize;
}

struct GatherParamPtrsVisitor<'a> {
    current_group_id: Option<GroupId>,
    params_vec: &'a mut Vec<(Option<GroupId>, ParamPtr)>,
    params_map: &'a mut FxHashMap<ParameterId, ParamPtr>,
    groups_vec: &'a mut Vec<(Option<GroupId>, *const dyn AnyParameterGroup)>,
}

impl GatherParamPtrsVisitor<'_> {
    fn add_param_ptr(&mut self, id: ParameterId, ptr: ParamPtr) {
        self.params_vec.push((self.current_group_id, ptr));
        self.params_map.insert(id, ptr);
    }
}

impl ParamVisitor for GatherParamPtrsVisitor<'_> {
    fn bool_parameter(&mut self, p: &super::BoolParameter) {
        self.add_param_ptr(p.id(), ParamPtr::Bool(NonNull::from_ref(p)));
    }

    fn bypass_parameter(&mut self, p: &super::ByPassParameter) {
        self.add_param_ptr(p.id(), ParamPtr::ByPass(NonNull::from_ref(p)));
    }

    fn float_parameter(&mut self, p: &super::FloatParameter) {
        self.add_param_ptr(p.id(), ParamPtr::Float(NonNull::from_ref(p)));
    }

    fn int_parameter(&mut self, p: &super::IntParameter) {
        self.add_param_ptr(p.id(), ParamPtr::Int(NonNull::from_ref(p)));
    }

    fn string_list_parameter(&mut self, p: &super::StringListParameter) {
        self.add_param_ptr(p.id(), ParamPtr::StringList(NonNull::from_ref(p)));
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
    params_vec: Vec<(Option<GroupId>, ParamPtr)>,
    params_map: FxHashMap<ParameterId, ParamPtr>,
    groups_vec: Vec<(Option<GroupId>, *const dyn AnyParameterGroup)>,
}

impl<P: Params> ParameterMap<P> {
    pub fn new(parameters: P) -> Rc<Self> {
        // Construct the instance first, so that parameters is moved into the correct memory location
        let mut this = Rc::new(Self {
            parameters,
            params_vec: Vec::new(),
            params_map: FxHashMap::default(),
            groups_vec: Vec::new(),
        });

        let this_ref = Rc::get_mut(&mut this).unwrap();
        let mut visitor = GatherParamPtrsVisitor {
            current_group_id: None,
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
    fn get_by_id<'a>(&'a self, id: ParameterId) -> Option<ParamRef<'a>> {
        self.params_map
            .get(&id)
            .map(|p| unsafe { ParamRef::from_param_ptr(*p) })
    }

    fn get_by_index<'s>(&'s self, index: usize) -> Option<(Option<GroupId>, ParamRef<'s>)> {
        self.params_vec
            .get(index)
            .map(|&(group_id, p)| (group_id, unsafe { ParamRef::from_param_ptr(p) }))
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
