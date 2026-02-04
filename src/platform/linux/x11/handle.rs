use std::{num::NonZeroU32, sync::Arc};

use raw_window_handle::{HandleError, RawDisplayHandle, RawWindowHandle, XcbWindowHandle};

use crate::{
    core::{PhysicalSize, Rect, ScaleFactor, WindowTheme},
    platform::{Error, linux::x11::X11Runloop},
};

pub struct X11Handle {
    pub runloop: Arc<X11Runloop>,
    pub id: u32,
}

impl X11Handle {
    pub fn global_bounds(&self) -> Rect {
        Rect::EMPTY
    }

    pub fn invalidate(&self, _rect: Rect) {
        self.runloop.get_window(self.id).needs_redraw.set(true);
    }

    pub fn invalidate_window(&self) {
        self.runloop.get_window(self.id).needs_redraw.set(true);
    }

    pub fn physical_size(&self) -> PhysicalSize {
        self.runloop.get_window(self.id).physical_size.get()
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        ScaleFactor(1.0)
    }

    pub fn theme(&self) -> WindowTheme {
        WindowTheme::Dark
    }

    pub fn get_clipboard(&self) -> Result<Option<String>, Error> {
        Ok(None)
    }

    pub fn set_clipboard(&self, _str: &str) -> Result<(), Error> {
        Ok(())
    }

    pub fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
        let handle = XcbWindowHandle::new({
            NonZeroU32::new(self.id).expect("XCB window id should be non-zero")
        });
        Ok(RawWindowHandle::Xcb(handle))
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        self.runloop.raw_display_handle()
    }
}
