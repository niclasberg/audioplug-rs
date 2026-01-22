use raw_window_handle::{HandleError, RawDisplayHandle, RawWindowHandle};

use crate::core::{PhysicalSize, Rect, ScaleFactor, WindowTheme, Zero};

pub struct Handle;

impl Handle {
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

    pub fn get_clipboard(&self) -> Result<Option<String>, super::Error> {
        Ok(None)
    }

    pub fn set_clipboard(&self, str: &str) -> Result<(), super::Error> {
        Ok(())
    }

    pub fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
        todo!()
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        todo!()
    }
}
