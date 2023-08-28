use std::cell::RefCell;

use crate::{widget::Widget, event::Event};

use super::Renderer;

pub(crate) struct WindowState {
	widget: RefCell<Box<dyn Widget>>
}

impl WindowState {
	pub(crate) fn new(widget: impl Widget + 'static) -> Self {
		Self { widget: RefCell::new(Box::new(widget)) }
	}

	pub(crate) fn dispatch_event(&self, event: Event) {
		self.widget.borrow_mut().event(event);
	}

	pub(crate) fn render<'a>(&self, renderer: &'a mut Renderer<'a>) {
		let mut wrapped_renderer = crate::window::Renderer(renderer);
		self.widget.borrow_mut().render(&mut wrapped_renderer);
	}
}