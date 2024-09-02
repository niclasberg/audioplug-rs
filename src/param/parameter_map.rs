use std::{any::Any, collections::HashMap};

use super::{ParamRef, ParameterId};

pub type ParameterGetter<P> = fn(&P) -> ParamRef;

pub trait Params: Default + 'static + Any {
    const PARAMS: &'static [ParameterGetter<Self>];
}

impl Params for () {
    const PARAMS: &'static [ParameterGetter<Self>] = &[];
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
	fn get_by_id(&self, id: ParameterId) -> Option<ParamRef>;
	fn get_by_index(&self, index: usize) -> Option<ParamRef>;
	fn count(&self) -> usize;
}

pub struct ParamIter<'a, P: Params + 'a> {
	parameters: &'a P,
	inner_iter: std::slice::Iter<'a, ParameterGetter<P>>
}

impl<'a, P: Params> Iterator for ParamIter<'a, P> {
	type Item = ParamRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner_iter.next().map(|getter|
			getter(&self.parameters))
	}
}

pub struct ParameterMap<P: Params + Any> {
	parameters: P,
	getters_map: HashMap<ParameterId, ParameterGetter<P>>
}

impl<P: Params + Any> ParameterMap<P> {
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
		ParamIter {
			parameters: &self.parameters,
			inner_iter: P::PARAMS.iter()
		}
	}
}

impl<P: Params + Any> AnyParameterMap for ParameterMap<P> {
	fn get_by_id(&self, id: ParameterId) -> Option<ParamRef> {
		self.getters_map.get(&id).map(|getter| {
			getter(&self.parameters)
		}) 
	}

	fn get_by_index(&self, index: usize) -> Option<ParamRef> {
		P::PARAMS.get(index).map(|getter| {
			getter(&self.parameters)
		}) 
	}

	fn count(&self) -> usize {
		P::PARAMS.len()
	}
}