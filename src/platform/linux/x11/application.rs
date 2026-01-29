use x11rb::xcb_ffi::XCBConnection;

pub struct X11Application {
    connection: XCBConnection
}

impl X11Application {
    pub fn new(connection: XCBConnection) -> Self {
        Self { connection }
    }
}