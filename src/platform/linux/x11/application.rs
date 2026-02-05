use std::sync::Arc;

use x11rb::connection::Connection;

use crate::platform::linux::x11::runloop::X11Runloop;

#[derive(Debug, PartialEq, Eq)]
enum Action {
    None,
    Close,
}

pub struct X11Application {
    runloop: Arc<X11Runloop>,
}

impl X11Application {
    pub fn new(runloop: Arc<X11Runloop>) -> Self {
        Self { runloop }
    }

    pub fn runloop(&self) -> Arc<X11Runloop> {
        self.runloop.clone()
    }

    pub fn run(&mut self) {
        loop {
            let event = self.runloop.connection.wait_for_event().unwrap();
            if self.handle_event(event) == Action::Close {
                return;
            }
            // Poll more, to not uneccesarily repaint
            while let Some(event) = self.runloop.connection.poll_for_event().unwrap() {
                if self.handle_event(event) == Action::Close {
                    return;
                }
            }
            self.runloop.repaint_if_requested();
        }
    }

    fn handle_event(&mut self, event: x11rb::protocol::Event) -> Action {
        use x11rb::protocol::Event;
        match &event {
            Event::ClientMessage(event) => {
                let data = event.data.as_data32();
                if event.format == 32 && data[0] == self.runloop.atoms.WM_DELETE_WINDOW {
                    return Action::Close;
                }
            }
            _ => {}
        };
        self.runloop.handle_event(event);
        Action::None
    }
}
