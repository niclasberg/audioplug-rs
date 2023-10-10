use std::cell::RefCell;

use crate::{event::Event, window::WindowHandler};

pub(crate) struct WindowState {
	handler: RefCell<Box<dyn WindowHandler>>
}

impl WindowState {
	pub(crate) fn new(handler: impl WindowHandler + 'static) -> Self {
		Self { handler: RefCell::new(Box::new(handler)) }
	}

	pub(crate) fn dispatch_event(&self, event: Event) {
		self.handler.borrow_mut().event(event);
	}
}