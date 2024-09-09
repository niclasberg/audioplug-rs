use std::{any::Any, collections::HashMap};

use super::{ParamRef, ParameterId};

pub type ParameterGetter<P> = fn(&P) -> ParamRef;
pub type ParameterGetterAny<P> = fn(&P) -> &dyn Any;

pub trait Params: Default + 'static {
    const PARAMS: &'static [(ParameterGetter<Self>, ParameterGetterAny<Self>)];
}

impl Params for () {
    const PARAMS: &'static [(ParameterGetter<Self>, ParameterGetterAny<Self>)] = &[];
}

/// A collection of parameters. Supports getting parameters by index and id
pub trait AnyParameterMap: 'static {
	fn get_by_id<'s>(&'s self, id: ParameterId) -> Option<ParamRef<'s>>;
	fn get_by_id_as_any(&self, id: ParameterId) -> Option<&dyn Any>;
	fn get_by_index(&self, index: usize) -> Option<ParamRef>;
	fn count(&self) -> usize;
}

pub struct ParamIter<'a, P: Params + 'a> {
	parameters: &'a P,
	inner_iter: std::slice::Iter<'a, (ParameterGetter<P>, ParameterGetterAny<P>)>
}

impl<'a, P: Params> Iterator for ParamIter<'a, P> {
	type Item = ParamRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner_iter.next().map(|getter|
			getter.0(&self.parameters))
	}
}

pub struct ParameterMap<P: Params> {
	parameters: P,
	getters_map: HashMap<ParameterId, ParameterGetter<P>>,
	getters_any_map: HashMap<ParameterId, ParameterGetterAny<P>>
}

impl<P: Params> ParameterMap<P> {
	pub fn new(parameters: P) -> Self {
		let getters_map = P::PARAMS.iter()
			.map(|getter| (getter.0(&parameters).id(), getter.0))
			.collect();

		let getters_any_map = P::PARAMS.iter()
			.map(|getter| (getter.0(&parameters).id(), getter.1))
			.collect();
	
		Self {
			parameters,
			getters_map,
			getters_any_map
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

impl<P: Params> AnyParameterMap for ParameterMap<P> {
	fn get_by_id(&self, id: ParameterId) -> Option<ParamRef> {
		self.getters_map.get(&id).map(|getter| {
			getter(&self.parameters)
		}) 
	}

	fn get_by_id_as_any(&self, id: ParameterId) -> Option<&dyn Any> {
		self.getters_any_map.get(&id).map(|getter| {
			getter(&self.parameters)
		}) 
	}

	fn get_by_index(&self, index: usize) -> Option<ParamRef> {
		P::PARAMS.get(index).map(|getter| {
			getter.0(&self.parameters)
		}) 
	}

	fn count(&self) -> usize {
		P::PARAMS.len()
	}
}