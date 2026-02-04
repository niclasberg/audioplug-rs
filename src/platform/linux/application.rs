use std::sync::Arc;

use crate::platform::linux::{
    wayland::application::WaylandApplication,
    x11::{X11Application, X11Runloop},
};

pub enum Application {
    Wayland(WaylandApplication),
    X11(X11Application),
}

impl Application {
    pub fn new() -> Self {
        if false && let Ok(connection) = wayland_client::Connection::connect_to_env() {
            Self::Wayland(WaylandApplication::new(connection))
        } else {
            let runloop = Arc::new(X11Runloop::new().unwrap());
            // Default to X11
            Self::X11(X11Application::new(runloop))
        }
    }

    pub fn run(&mut self) {
        match self {
            Application::Wayland(app) => app.run(),
            Application::X11(app) => app.run(),
        }
    }
}
