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