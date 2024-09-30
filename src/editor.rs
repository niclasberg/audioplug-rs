use std::marker::PhantomData;

use crate::{app::AppContext, core::Size, param::{AnyParameter, ParamRef, Params}, view::{AnyView, Column, Label, ParameterSlider, Row, View}};

pub trait Editor: 'static {
	type Parameters: Params;

	fn new() -> Self;
    fn view(&self, ctx: &mut AppContext, parameters: &Self::Parameters) -> AnyView;
	fn min_size(&self) -> Option<Size> { None }
	fn max_size(&self) -> Option<Size> { None }
	fn prefered_size(&self) -> Option<Size> { None }
}

pub struct GenericEditor<P> {
	_phantom: PhantomData<P>
}

impl<P: Params> Editor for GenericEditor<P> {
	type Parameters = P;
	
	fn new() -> Self {
		Self { _phantom: PhantomData }
	}
	
    fn view(&self, _ctx: &mut AppContext, parameters: &P) -> AnyView {
		let mut views: Vec<AnyView> = Vec::new();
		/*for param in parameters.visit() {
			match param {
				ParamRef::Float(p) => {
					let view = Row::new((
						Label::new(p.info().name()),
						ParameterSlider::new(p).as_any(),
					));
					views.push(view.as_any());
				},
				ParamRef::Int(_) => todo!(),
				ParamRef::StringList(_) => todo!(),
				ParamRef::ByPass(_) => todo!(),
				ParamRef::Bool(_) => todo!(),
			}
		}*/
        Column::new(views).as_any()
    }
}