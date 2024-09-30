use super::{group::{AnyParameterGroup, ParamGroup}, BoolParameter, ByPassParameter, FloatParameter, IntParameter, ParamRef, ParameterGroup, StringListParameter};

#[macro_export] 
macro_rules! params {
	($(#[$struct_meta:meta])*
	$sv:vis struct $name:ident { $($fv:vis $fname:ident : $ftype:ty), * }
	) => {
		$(#[$struct_meta])*
        $sv struct $name {
            $($fv $fname: $ftype,)*
        }

		impl $crate::param::ParameterTraversal for $name {
			fn visit<V: ParamVisitor>(&self, visitor: &V) -> V::Value {
				$($crate::param::VisitParameter::visit(&this.$fname, visitor),)*
			}
		}
	}
}

pub trait ParamVisitor {
	fn bool_parameter(&self, p: &BoolParameter);
	fn bypass_parameter(&self, p: &ByPassParameter);
	fn float_parameter(&self, p: &FloatParameter);
	fn int_parameter(&self, p: &IntParameter);
	fn string_list_parameter(&self, p: &StringListParameter);
	fn group<P: ParamGroup>(&self, group: &ParameterGroup<P>);
}

pub trait ParameterTraversal {
	fn visit<V: ParamVisitor>(&self, visitor: &V);
}

impl ParameterTraversal for () {
	fn visit<V: super::param_lens::ParamVisitor>(&self, _visitor: &V) {}
}