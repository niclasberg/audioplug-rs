use std::{collections::HashMap, rc::Rc};

use crate::param::ParamVisitor;

use super::{group::AnyParameterGroup, traversal::ParameterTraversal, AnyParameter, GroupId, ParamRef, ParameterId};


pub trait Params: ParameterTraversal {
    fn new() -> Self;
}

impl Params for () { 
	fn new() -> Self {
		()
	}
}

pub struct ParamIter<'a> {
	inner_iter: std::slice::Iter<'a, (Option<GroupId>, *const dyn AnyParameter)>
}

impl<'a> Iterator for ParamIter<'a> {
	type Item = ParamRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner_iter.next().map(|&(_, p)| unsafe { &*p }.as_param_ref() )
	}
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
	fn get_by_id(&self, id: ParameterId) -> Option<&dyn AnyParameter>;
	fn get_by_index(&self, index: usize) -> Option<(Option<GroupId>, &dyn AnyParameter)>;
	fn count(&self) -> usize;
	fn parameter_ids(&self) -> Vec<ParameterId>;
	fn get_group_by_index(&self, index: usize) -> Option<(Option<GroupId>, &dyn AnyParameterGroup)>;
	fn groups_count(&self) -> usize;
}

struct GatherParamPtrsVisitor<'a> {
	current_group_id: Option<GroupId>,
	params_vec: &'a mut Vec<(Option<GroupId>, *const dyn AnyParameter)>,
	params_map: &'a mut HashMap<ParameterId, *const dyn AnyParameter>,
	groups_vec: &'a mut Vec<(Option<GroupId>, *const dyn AnyParameterGroup)>,
}

impl<'a> GatherParamPtrsVisitor<'a> {
	fn add_param_ptr(&mut self, p: &dyn AnyParameter) {
		let id = p.info().id();
		let param_ptr = p as *const dyn AnyParameter;
		self.params_vec.push((self.current_group_id, param_ptr));
		self.params_map.insert(id, param_ptr);
	}
}

impl<'a> ParamVisitor for GatherParamPtrsVisitor<'a> {
	fn bool_parameter(&mut self, p: &super::BoolParameter) {
		self.add_param_ptr(p);
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
		self.groups_vec.push((self.current_group_id, group as *const _));

		let old_group_id = self.current_group_id;
		self.current_group_id = Some(group.id());
		group.children().visit(self);
		self.current_group_id = old_group_id;
	}
}

pub struct ParameterMap<P: Params> {
	parameters: P,
	params_vec: Vec<(Option<GroupId>, *const dyn AnyParameter)>,
	params_map: HashMap<ParameterId, *const dyn AnyParameter>,
	groups_vec: Vec<(Option<GroupId>, *const dyn AnyParameterGroup)>,
}

impl<P: Params> ParameterMap<P> {
	pub fn new(parameters: P) -> Rc<Self> {
		// Construct the instance first, so that parameters is moved into the correct memory location
		let mut this = Rc::new(Self {
			parameters: parameters,
			params_vec: Vec::new(),
			params_map: HashMap::new(),
			groups_vec: Vec::new()
		});

		let this_ref = Rc::get_mut(&mut this).unwrap();
		let mut visitor = GatherParamPtrsVisitor {
			current_group_id: None,
			params_vec: &mut this_ref.params_vec,
			params_map: &mut this_ref.params_map,
			groups_vec: &mut this_ref.groups_vec
		};
		this_ref.parameters.visit(&mut visitor);

		this
	}

	pub fn parameters_ref(&self) -> &P {
		&self.parameters
	}

	pub fn iter<'s>(&'s self) -> ParamIter<'s> {
		ParamIter {
    		inner_iter: self.params_vec.as_slice().iter(),
		}
	}
}

impl<P: Params> AnyParameterMap for ParameterMap<P> {
	fn get_by_id(&self, id: ParameterId) -> Option<&dyn AnyParameter> {
		self.params_map.get(&id)
			.map(|&p| unsafe { &*p} ) 
	}

	fn get_by_index(&self, index: usize) -> Option<(Option<GroupId>, &dyn AnyParameter)> {
		self.params_vec.get(index)
			.map(|&(group_id, p)| (group_id, unsafe { &*p })) 
	}

	fn count(&self) -> usize {
		self.params_vec.len()
	}

	fn parameter_ids(&self) -> Vec<ParameterId> {
		self.iter()
			.map(|param_ref| param_ref.id())
			.collect()
	}

	fn groups_count(&self) -> usize {
		self.groups_vec.len()
	}

	fn get_group_by_index(&self, index: usize) -> Option<(Option<GroupId>, &dyn AnyParameterGroup)> {
		self.groups_vec.get(index)
			.map(|&(group_id, g)| (group_id, unsafe { &*g }))
	}
}