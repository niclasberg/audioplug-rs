use std::collections::HashMap;

use crate::param::ParamVisitor;

use super::{group::AnyParameterGroup, param_lens::ParameterTraversal, AnyParameter, GroupId, ParamRef, ParameterId};


pub type ParameterGetter<P> = fn(&P) -> ParamRef;


pub trait Params: ParameterTraversal + 'static {
    fn new() -> Self;
}

impl Params for () { 
	fn new() -> Self {
		()
	}
}

pub struct ParamIter<'a> {
	inner_iter: std::slice::Iter<'a, *const dyn AnyParameter>
}

impl<'a> Iterator for ParamIter<'a> {
	type Item = ParamRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner_iter.next().map(|p| unsafe { &**p }.as_param_ref() )
	}
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
	fn get_by_id(&self, id: ParameterId) -> Option<&dyn AnyParameter>;
	fn get_group_id(&self, id: ParameterId) -> Option<GroupId>;
	fn get_by_index(&self, index: usize) -> Option<&dyn AnyParameter>;
	fn count(&self) -> usize;
	fn parameter_ids(&self) -> Vec<ParameterId>;
}

struct GatherParamPtrsVisitor<'a> {
	current_group_id: Option<GroupId>,
	params_vec: &'a mut Vec<*const dyn AnyParameter>,
	params_map: &'a mut HashMap<ParameterId, *const dyn AnyParameter>,
	params_group_ids: &'a mut HashMap<ParameterId, GroupId>,
}

impl<'a> GatherParamPtrsVisitor<'a> {
	fn add_param_ptr(&mut self, p: &dyn AnyParameter) {
		let id = p.info().id();
		let param_ptr = p as *const dyn AnyParameter;
		self.params_vec.push(param_ptr);
		self.params_map.insert(id, param_ptr);
		if let Some(group_id) = self.current_group_id {
			self.params_group_ids.insert(id, group_id);
		}
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
		let old_group_id = self.current_group_id;
		self.current_group_id = Some(group.id());
		group.children().visit(self);
		self.current_group_id = old_group_id;
	}
}

pub struct ParameterMap<P: Params> {
	parameters: P,
	params_vec: Vec<*const dyn AnyParameter>,
	params_map: HashMap<ParameterId, *const dyn AnyParameter>,
	params_group_ids: HashMap<ParameterId, GroupId>,
}

impl<P: Params> ParameterMap<P> {
	pub fn new(parameters: P) -> Self {
		// Construct the instance first, so that parameters is moved into the correct memory location
		let mut this = Self {
			parameters,
			params_vec: Vec::new(),
			params_map: HashMap::new(),
			params_group_ids: HashMap::new()
		};

		let mut visitor = GatherParamPtrsVisitor {
			current_group_id: None,
			params_vec: &mut this.params_vec,
			params_map: &mut this.params_map,
			params_group_ids: &mut this.params_group_ids,
		};
		this.parameters.visit(&mut visitor);

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

	fn get_group_id(&self, id: ParameterId) -> Option<GroupId> {
		self.params_group_ids.get(&id)
			.map(|group_id| *group_id)
	}

	fn get_by_index(&self, index: usize) -> Option<&dyn AnyParameter> {
		self.params_vec.get(index)
			.map(|&p| unsafe { &*p }) 
	}

	fn count(&self) -> usize {
		self.params_vec.len()
	}

	fn parameter_ids(&self) -> Vec<ParameterId> {
		self.iter()
			.map(|param_ref| param_ref.id())
			.collect()
	}
}