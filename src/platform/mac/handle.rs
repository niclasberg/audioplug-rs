use crate::core::Rectangle;
use super::view::View;

pub struct HandleRef<'a> {
	view: &'a View
}

impl<'a> HandleRef<'a> {
	pub(crate) fn new(view: &'a View) -> Self {
		Self { view }
	}

	pub fn global_bounds(&self) -> Rectangle {
		unsafe { self.view.bounds().into() }
	}

	pub fn invalidate(&self, rect: Rectangle) {
		println!("Invalidate: {:?}", rect);
		unsafe { self.view.setNeedsDisplayInRect(rect.into()) }
	}
}