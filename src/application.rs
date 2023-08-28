#[cfg(target_os = "windows")]
use crate::win as platform;

pub struct Application(platform::Application);

impl Application {
    pub fn new() -> Self {
        Self(platform::Application::new())
    }

    pub fn run(&mut self) {
        self.0.run()
    }
}