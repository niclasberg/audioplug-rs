use std::sync::Arc;

use x11rb::connection::Connection;

use crate::platform::linux::x11::runloop::X11Runloop;

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
        use x11rb::protocol::Event;
        loop {
            let event = self.runloop.connection.wait_for_event().unwrap();
            match &event {
                Event::ClientMessage(event) => {
                    let data = event.data.as_data32();
                    if event.format == 32 && data[0] == self.runloop.atoms.WM_DELETE_WINDOW {
                        break;
                    }
                }
                _ => {}
            };
            self.runloop.handle_event(event);
        }
    }
}
