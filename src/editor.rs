use std::marker::PhantomData;

use crate::{app::AppContext, core::{Color, Point, Rectangle, Size}, param::Params, view::{AnyView, Fill, View}};

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
	
    fn view(&self, _ctx: &mut AppContext, _parameters: &P) -> AnyView {
        Rectangle::new(Point::ZERO, Size::new(200.0, 200.0)).fill(Color::RED).as_any()
    }
}