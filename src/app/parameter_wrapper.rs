use std::{any::Any, collections::HashMap};

use crate::param::{ParamRef, ParameterGetter, ParameterId, Params};

pub trait ParameterWrapper: 'static {
	fn get_by_id(&self, id: ParameterId) -> Option<ParamRef>;
	fn get_by_index(&self, index: usize) -> Option<ParamRef>;
	fn count(&self) -> usize;
}

pub(super) fn wrap_parameters<P: Params + Any>(parameters: P) -> Box<dyn ParameterWrapper> {
	let getters_map = P::PARAMS.iter()
		.map(|getter| (getter(&parameters).id(), *getter))
		.collect();
	
	let wrapper = ParameterWrapperImpl {
		parameters,
		getters_map
	};
	Box::new(wrapper)
}

struct ParameterWrapperImpl<P: Params + Any> {
	parameters: P,
	getters_map: HashMap<ParameterId, ParameterGetter<P>>
}

impl<P: Params + Any> ParameterWrapper for ParameterWrapperImpl<P> {
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