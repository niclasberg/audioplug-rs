use std::collections::HashMap;

use super::{group::AnyParameterGroup, param_lens::ParameterTraversal, GroupId, ParamRef, ParameterId};


pub type ParameterGetter<P> = fn(&P) -> ParamRef;


pub trait Params: ParameterTraversal + 'static {
    fn new() -> Self;
}

impl Params for () { 
	fn new() -> Self {
		()
	}
}

pub struct ParamIter<'a, P: Params + 'a> {
	parameters: &'a P,
	inner_iter: std::slice::Iter<'a, ParameterGetter<P>>
}

impl<'a, P: Params> Iterator for ParamIter<'a, P> {
	type Item = ParamRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner_iter.next().map(|getter| getter(&self.parameters))
	}
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
	fn get_by_id<'s>(&'s self, id: ParameterId) -> Option<ParamRef<'s>>;
	fn get_group_id(&self, id: ParameterId) -> Option<GroupId>;
	fn get_by_index(&self, index: usize) -> Option<ParamRef>;
	fn count(&self) -> usize;
	fn parameter_ids(&self) -> Vec<ParameterId>;
}

pub struct ParameterMap<P: Params> {
	parameters: P,
	getters_map: HashMap<ParameterId, ParameterGetter<P>>,
}

impl<P: Params> ParameterMap<P> {
	pub fn new(parameters: P) -> Self {
		let getters_map = P::PARAMS.iter()
			.map(|getter| (getter(&parameters).id(), *getter))
			.collect();
	
		Self {
			parameters,
			getters_map
		}
	}

	pub fn parameters_ref(&self) -> &P {
		&self.parameters
	}

	pub fn iter<'s>(&'s self) -> ParamIter<'s, P> {
		self.parameters.param_ref_iter()
	}
}

impl<P: Params> AnyParameterMap for ParameterMap<P> {
	fn get_by_id(&self, id: ParameterId) -> Option<ParamRef> {
		self.getters_map.get(&id).map(|getter| {
			getter(&self.parameters)
		}) 
	}

	fn get_group_id(&self, _id: ParameterId) -> Option<GroupId> {
		None
	}

	fn get_by_index(&self, index: usize) -> Option<ParamRef> {
		P::PARAMS.get(index).map(|getter| {
			getter(&self.parameters)
		}) 
	}

	fn count(&self) -> usize {
		P::PARAMS.len()
	}

	fn parameter_ids(&self) -> Vec<ParameterId> {
		self.iter()
			.map(|param_ref| param_ref.id())
			.collect()
	}
}