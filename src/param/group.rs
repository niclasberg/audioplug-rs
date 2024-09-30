use super::{param_lens::{ParamVisitor, ParameterTraversal}, GroupId,  ParameterId};

pub trait ParamGroup: ParameterTraversal {
	fn new_with_offset(offset: ParameterId) -> Self;
}

pub trait AnyParameterGroup {
	fn id(&self) -> GroupId;
	fn name(&self) -> &'static str;
}

pub struct ParameterGroup<P: ParamGroup> {
	id: GroupId,
	name: &'static str,
	children: P
}

impl<P: ParamGroup> ParameterGroup<P> {

}

impl<P: ParamGroup> AnyParameterGroup for ParameterGroup<P> {
	fn id(&self) -> GroupId {
		self.id
	}

	fn name(&self) -> &'static str {
		&self.name
	}
}

impl<P: ParamGroup> ParameterTraversal for ParameterGroup<P> {
	fn visit<V: ParamVisitor>(&self, visitor: &V) -> V::Value {
		visitor.parameter_group(self)
	}
}