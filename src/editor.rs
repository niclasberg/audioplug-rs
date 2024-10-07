use std::marker::PhantomData;

use crate::{app::ViewContext, core::Size, param::{AnyParameter, AnyParameterGroup, ParamVisitor, ParameterTraversal, Params}, view::{AnyView, Column, Label, ParameterSlider, Row, View}};

pub trait Editor: 'static {
	type Parameters: Params;

	fn new() -> Self;
    fn view(&self, ctx: &mut ViewContext, parameters: &Self::Parameters) -> AnyView;
	fn min_size(&self) -> Option<Size> { None }
	fn max_size(&self) -> Option<Size> { None }
	fn prefered_size(&self) -> Option<Size> { None }
}

struct CreateParameterViewsVisitor {
	views: Vec<AnyView>
}

impl CreateParameterViewsVisitor {
	pub fn new() -> Self {
		Self {
			views: Vec::new()
		}
	}
}

impl ParamVisitor for CreateParameterViewsVisitor {
	fn bool_parameter(&mut self, p: &crate::param::BoolParameter) {
		
	}

	fn bypass_parameter(&mut self, p: &crate::param::ByPassParameter) {
		
	}

	fn float_parameter(&mut self, p: &crate::param::FloatParameter) {
		let view = Row::new((
			Label::new(p.info().name()),
			ParameterSlider::new(p).as_any(),
		));
		self.views.push(view.as_any());
	}

	fn int_parameter(&mut self, p: &crate::param::IntParameter) {
		let view = Row::new((
			Label::new(p.info().name()),
			ParameterSlider::new(p).as_any(),
		));
		self.views.push(view.as_any());
	}

	fn string_list_parameter(&mut self, p: &crate::param::StringListParameter) {
		
	}

	fn group<P: ParameterTraversal>(&mut self, group: &crate::param::ParameterGroup<P>) {
		let mut child_visitor = Self::new();
		group.children().visit(&mut child_visitor);

		let view = Column::new((
			Label::new(group.name()),
			Column::new(child_visitor.views)
				.padding_left(20.0)
		)).padding_top(10.0);
		self.views.push(view.as_any());
	}
}

pub struct GenericEditor<P> {
	_phantom: PhantomData<P>
}

impl<P: Params> Editor for GenericEditor<P> {
	type Parameters = P;
	
	fn new() -> Self {
		Self { _phantom: PhantomData }
	}
	
    fn view(&self, _ctx: &mut ViewContext, parameters: &P) -> AnyView {
		let mut visitor = CreateParameterViewsVisitor::new();
		parameters.visit(&mut visitor);
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
        Column::new(visitor.views).as_any()
    }
}