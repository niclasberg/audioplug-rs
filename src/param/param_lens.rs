use super::{group::AnyParameterGroup, BoolParameter, ByPassParameter, FloatParameter, IntParameter, ParamRef, ParameterGroup, StringListParameter};

#[macro_export] 
macro_rules! params {
	($(#[$struct_meta:meta])*
	$sv:vis struct $name:ident { $($fv:vis $fname:ident : $ftype:ty), * }
	) => {
		$(#[$struct_meta])*
        $sv struct $name {
            $(
				$fv $fname: $ftype
			),*
        }

		impl $crate::param::ParameterTraversal for $name {
			fn visit<V: $crate::param::ParamVisitor>(&self, visitor: &mut V) {
				$(
					$crate::param::ParameterTraversal::visit(&self.$fname, visitor);
				)*
			}
		}
	}
}

pub trait ParamVisitor {
	fn bool_parameter(&mut self, p: &BoolParameter);
	fn bypass_parameter(&mut self, p: &ByPassParameter);
	fn float_parameter(&mut self, p: &FloatParameter);
	fn int_parameter(&mut self, p: &IntParameter);
	fn string_list_parameter(&mut self, p: &StringListParameter);
	fn group<P: ParameterTraversal>(&mut self, group: &ParameterGroup<P>);
}

pub trait ParameterTraversal: 'static {
	fn visit<V: ParamVisitor>(&self, visitor: &mut V);
}

impl ParameterTraversal for () {
	fn visit<V: super::param_lens::ParamVisitor>(&self, _visitor: &mut V) {}
}