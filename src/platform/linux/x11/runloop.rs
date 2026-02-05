use std::{
    os::fd::{AsRawFd, RawFd},
    ptr::NonNull,
};

use atomic_refcell::{AtomicRef, AtomicRefCell};
use raw_window_handle::{HandleError, RawDisplayHandle, XcbDisplayHandle};
use x11rb::{connection::Connection, protocol::xproto::Screen, xcb_ffi::XCBConnection};

use crate::{
    core::FxHashMap,
    platform::{Error, linux::x11::window::WindowInner},
};

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
    }
}

pub struct X11Runloop {
    pub(super) connection: XCBConnection,
    pub(super) screen_id: usize,
    windows: AtomicRefCell<FxHashMap<u32, WindowInner>>,
    pub atoms: Atoms,
}

impl X11Runloop {
    pub fn new() -> Result<Self, Error> {
        x11rb::xcb_ffi::load_libxcb().map_err(|e| Error::from_reason(e))?;
        let (connection, screen_id) =
            x11rb::xcb_ffi::XCBConnection::connect(None).map_err(|e| Error::from_reason(e))?;

        let atoms = Atoms::new(&connection)
            .map_err(|e| Error::from_reason(e))?
            .reply()
            .map_err(|e| Error::from_reason(e))?;

        Ok(Self {
            connection,
            screen_id,
            windows: Default::default(),
            atoms,
        })
    }

    pub(super) fn register_window(&self, id: u32, inner: WindowInner) {
        let mut windows = self.windows.borrow_mut();
        windows.insert(id, inner);
    }

    pub(super) fn unregister_window(&self, id: u32) {
        let mut windows = self.windows.borrow_mut();
        windows.remove(&id);
    }

    pub fn get_window<'s>(&'s self, id: u32) -> AtomicRef<'s, WindowInner> {
        AtomicRef::map(self.windows.borrow(), |w| w.get(&id).unwrap())
    }

    pub(super) fn screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_id]
    }

    pub fn raw_fd(&self) -> RawFd {
        self.connection.as_raw_fd()
    }

    pub fn poll(&self) -> Result<(), Error> {
        if let Some(ev) = self
            .connection
            .poll_for_event()
            .map_err(|e| Error::from_reason(e))?
        {
            self.handle_event(ev);
        }
        Ok(())
    }

    pub fn handle_event(&self, event: x11rb::protocol::Event) {
        use x11rb::protocol::Event;
        let windows = self.windows.borrow();
        match event {
            Event::Expose(ev) => {
                windows
                    .get(&ev.window)
                    .expect("Window should exist when receiving expose event")
                    .handle_expose(ev);
            }
            Event::ButtonPress(ev) => {
                windows
                    .get(&ev.event)
                    .expect("Window should exist when receiving button press event")
                    .handle_button_press(ev);
            }
            Event::ButtonRelease(ev) => {
                windows
                    .get(&ev.event)
                    .expect("Window should exist when receiving button release event")
                    .handle_button_release(ev);
            }
            Event::MotionNotify(ev) => {
                windows
                    .get(&ev.event)
                    .expect("Window should exist when receiving mouse motion event")
                    .handle_motion(ev);
            }
            Event::ConfigureNotify(ev) => {
                windows
                    .get(&ev.event)
                    .expect("Window should exist when receiving configure event")
                    .handle_configure(ev);
            }
            _ => {}
        }
    }

    pub fn repaint_if_requested(&self) {
        for window in self.windows.borrow().values() {
            window.repaint_if_requested();
        }
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        let handle = XcbDisplayHandle::new(
            NonNull::new(self.connection.get_raw_xcb_connection()),
            self.screen_id as _,
        );
        Ok(RawDisplayHandle::Xcb(handle))
    }
}
