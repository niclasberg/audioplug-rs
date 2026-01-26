pub enum Application {
    Wayland {
        connection: wayland_client::Connection,
    },
    X11 {

    }
}

impl Application {
    pub fn new() -> Self {
        if let Ok(connection) = wayland_client::Connection::connect_to_env() {
            Self::Wayland { connection }
        } else {
            // Default to X11
            Self::X11 {  }
        }
    }

    pub fn run(&mut self) {
        match self {
            Application::Wayland { .. } =>  {
                
            },
            Application::X11 {  } => {},
        }
    }
}