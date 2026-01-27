use raw_window_handle::{WaylandWindowHandle, XcbWindowHandle};

use crate::{core::{PhysicalRect, Rect, ScaleFactor}, platform::{Error, WindowHandler, linux::wayland::application::WaylandApplication}};

pub struct WaylandWindow;

impl WaylandWindow {
    pub fn open(app: &mut WaylandApplication, _handler: Box<dyn WindowHandler>) -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn attach(_handle: WaylandWindowHandle, _handler: Box<dyn WindowHandler>) -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn set_scale_factor(&self, _scale_factor: ScaleFactor) {
        
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        ScaleFactor::default()
    }

    pub fn set_physical_size(&self, _size: PhysicalRect) -> Result<(), Error> {
        Ok(())
    }

    pub fn set_logical_size(&self, _rect: Rect) -> Result<(), Error> {
        Ok(())
    }
}