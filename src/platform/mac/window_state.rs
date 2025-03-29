use std::cell::RefCell;

use crate::platform::WindowHandler;

pub(crate) struct WindowState {
    handler: RefCell<Box<dyn WindowHandler>>,
}

impl WindowState {
    pub(crate) fn new(handler: impl WindowHandler + 'static) -> Self {
        Self {
            handler: RefCell::new(Box::new(handler)),
        }
    }
}
