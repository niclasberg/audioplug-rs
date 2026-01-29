use x11rb::xcb_ffi::XCBConnection;

pub struct X11Application {
    connection: XCBConnection,
    screen: u32,
    windows: FxHashMap<u32, Rc<WindowInner>>,
}

impl X11Application {
    pub fn new(connection: XCBConnection, screen: u32) -> Self {
        Self { connection, screen }
    }

    pub fn run(&mut self) {
        loop {
            self.connection.wait_for_event().unwrap()
        }
    }

    pub(super) fn register_window(&mut self, id: u32, inner: Rc<WindowInner>) {
        self.windows.insert(id, inner);
    }

    pub(super) fn unregister_window(&mut self, id: u32) {
        self.windows.erase(id);
    }
}
