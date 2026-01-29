use crate::platform::linux::{wayland::application::WaylandApplication, x11::X11Application};

pub enum Application {
    Wayland(WaylandApplication),
    X11(X11Application),
}

impl Application {
    pub fn new() -> Self {
        if let Ok(connection) = wayland_client::Connection::connect_to_env() {
            Self::Wayland(WaylandApplication::new(connection))
        } else {
            let (connection, screen) = x11rb::xcb_ffi::XCBConnection::connect(None).unwrap();
            // Default to X11
            Self::X11(X11Application::new(connection, screen))
        }
    }

    pub fn run(&mut self) {
        match self {
            Application::Wayland(app) => app.run(),
            Application::X11(app) => app.run(),
        }
    }
}
