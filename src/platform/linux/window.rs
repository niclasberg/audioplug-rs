use raw_window_handle::{WaylandWindowHandle, XcbWindowHandle};

use crate::{core::{PhysicalRect, Rect, ScaleFactor}, platform::WindowHandler};

pub struct Window;

impl Window {
    pub fn open(_handler: Box<dyn WindowHandler>) -> Result<Self, super::Error> {
        Ok(Self)
    }

    pub fn attach_wayland(_handle: WaylandWindowHandle, _handler: Box<dyn WindowHandler>) -> Result<Self, super::Error> {
        Ok(Self)
    }

    pub fn attach_xcb(_handle: XcbWindowHandle, _handler: Box<dyn WindowHandler>) -> Result<Self, super::Error> {
        Ok(Self)
    }

    pub fn set_scale_factor(&self, _scale_factor: ScaleFactor) {
        
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        ScaleFactor::default()
    }

    pub fn set_physical_size(&self, _size: PhysicalRect) -> Result<(), super::Error> {
        Ok(())
    }

    pub fn set_logical_size(&self, _rect: Rect) -> Result<(), super::Error> {
        Ok(())
    }
}