use std::rc::Rc;

use x11rb::{connection::Connection, protocol::xproto::Screen, xcb_ffi::XCBConnection};

use crate::core::FxHashMap;
use super::window::WindowInner;

pub struct X11Application {
    pub(super) connection: XCBConnection,
    screen_id: usize,
    windows: FxHashMap<u32, Rc<WindowInner>>,
}

impl X11Application {
    pub fn new(connection: XCBConnection, screen_id: usize) -> Self {
        Self { connection, screen_id, windows: Default::default() }
    }

    pub fn run(&mut self) {
        loop {
            self.connection.wait_for_event().unwrap();
        }
    }

    pub(super) fn register_window(&mut self, id: u32, inner: Rc<WindowInner>) {
        self.windows.insert(id, inner);
    }

    pub(super) fn unregister_window(&mut self, id: u32) {
        self.windows.remove(&id);
    }

    pub(super) fn screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_id]
    }
}
