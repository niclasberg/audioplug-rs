mod application;
mod audio;
mod executor;
mod handle;
mod text;
mod window;
mod wayland;

pub use application::Application;
pub use audio::{AudioHost, Device};
pub use executor::Executor;
pub use handle::Handle;
pub use text::{NativeFont, NativeTextLayout};
pub use window::Window;

use crate::core::{Size, Zero};

#[derive(Debug)]
pub struct Error;

pub struct Bitmap;

impl Bitmap {
    pub fn from_file(_path: &std::path::Path) -> Result<Self, super::Error> {
        Ok(Self)
    }

    pub fn size(&self) -> Size {
        Size::ZERO
    }
}

pub enum Handle {
    Wayland(WaylandHandle)
}

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

    pub fn set_clipboard(&self, _str: &str) -> Result<(), super::Error> {
        Ok(())
    }

    pub fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
        match self {
            Handle::Wayland(handle) => handle.raw_window_handle()
        }
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        match self {
            Handle::Wayland(handle) => handle.raw_display_handle()
        }
    }
}
