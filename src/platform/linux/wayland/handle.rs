use std::ptr::NonNull;

use raw_window_handle::{HandleError, RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use wayland_client::{Proxy, protocol::{wl_display::WlDisplay, wl_surface::WlSurface}};

use crate::{core::{PhysicalSize, Rect, ScaleFactor, WindowTheme, Zero}, platform::Error};

pub struct WaylandHandle {
    display: WlDisplay,
    surface: WlSurface,
}

impl WaylandHandle {
    pub fn global_bounds(&self) -> Rect {
        Rect::EMPTY
    }

    pub fn invalidate(&self, _rect: Rect) {

    }

    pub fn invalidate_window(&self) {

    }

    pub fn physical_size(&self) -> PhysicalSize {
        PhysicalSize::ZERO
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
        let handle: WaylandWindowHandle = WaylandWindowHandle::new({
            let ptr = self.surface.id().as_ptr();
            NonNull::new(ptr as *mut _).expect("Wayland surface pointer should never be null")
        });
        Ok(RawWindowHandle::Wayland(handle))
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        let handle = WaylandDisplayHandle::new({
            let ptr = self.display.id().as_ptr();
            NonNull::new(ptr as *mut _).expect("Wayland display pointer should never be null")
        });
        Ok(RawDisplayHandle::Wayland(handle))
    }
}
