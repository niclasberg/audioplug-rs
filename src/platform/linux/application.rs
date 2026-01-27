use crate::platform::linux::wayland::application::WaylandApplication;

pub enum Application {
    Wayland(WaylandApplication),
    X11
}

impl Application {
    pub fn new() -> Self {
        if let Ok(connection) = wayland_client::Connection::connect_to_env() {
            Self::Wayland(WaylandApplication::new(connection))
        } else {
            // Default to X11
            Self::X11 {  }
        }
    }

    pub fn run(&mut self) {
        match self {
            Application::Wayland(app) =>  {
                app.run()
            },
            Application::X11 {  } => {},
        }
    }
}