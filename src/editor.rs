use std::marker::PhantomData;

use crate::{core::{Color, Point, Rectangle, Size}, param::Params, view::{Fill, View}};

pub trait Editor<P: Params> {
    fn view(&self, parameters: &P) -> impl View;
	fn min_size() -> Option<Size> { None }
	fn max_size() -> Option<Size> { None }
	fn prefered_size() -> Option<Size> { None }
}

pub struct GenericEditor<P> {
	_phantom: PhantomData<P>
}

impl<P: Params> GenericEditor<P> {
	pub fn new() -> Self {
		Self {
			_phantom: PhantomData
		}
	}
}

impl<P: Params> Editor<P> for GenericEditor<P> {
    fn view(&self, _parameters: &P) -> impl View {
        Rectangle::new(Point::ZERO, Size::new(200.0, 200.0)).fill(Color::RED)
    }
}