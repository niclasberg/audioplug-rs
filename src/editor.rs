use std::marker::PhantomData;

use crate::{core::{Color, Point, Rectangle, Size}, param::Params, view::{AnyView, Fill, View}, window::AppContext};

pub trait Editor<P: Params>: 'static {
	fn new() -> Self;
    fn view(&self, ctx: &mut AppContext, parameters: &P) -> AnyView;
	fn min_size(&self) -> Option<Size> { None }
	fn max_size(&self) -> Option<Size> { None }
	fn prefered_size(&self) -> Option<Size> { None }
}

pub struct GenericEditor<P> {
	_phantom: PhantomData<P>
}

impl<P: Params> Editor<P> for GenericEditor<P> {
	fn new() -> Self {
		Self { _phantom: PhantomData }
	}
	
    fn view(&self, ctx: &mut AppContext, parameters: &P) -> AnyView {
        Rectangle::new(Point::ZERO, Size::new(200.0, 200.0)).fill(Color::RED).as_any()
    }
}