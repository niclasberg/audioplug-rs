#[macro_export] 
macro_rules! params {
	($(#[$struct_meta:meta])*
	$sv:vis struct $name:ident { $($fv:vis $fname:ident : $ftype:ty), * }
	) => {
		$(#[$struct_meta])*
        $sv struct $name {
            $($fv $fname: $ftype,)*
        }

		impl $crate::param::Params for $name {
			const PARAMS: &'static [(fn(&Self) -> $crate::param::ParamRef, fn(&Self) -> &dyn std::any::Any)] = &[
				$((|this| this.$fname.as_param_ref(), |this| this.$fname.as_any()),)*
			];
		}
	}
}