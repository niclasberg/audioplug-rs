mod application;
mod audio;
mod executor;
mod text;
mod window;
mod wayland;

pub use application::Application;
pub use audio::{AudioHost, Device};
pub use executor::Executor;
use raw_window_handle::{HandleError, RawDisplayHandle, RawWindowHandle};
pub use text::{NativeFont, NativeTextLayout};
pub use window::Window;

use crate::{core::{PhysicalSize, Rect, ScaleFactor, Size, WindowTheme, Zero}, platform::linux::wayland::handle::WaylandHandle};

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
        match self {
            Handle::Wayland(handle) => handle.global_bounds(),
        }
    }

    pub fn invalidate(&self, rect: Rect) {
        match self {
            Handle::Wayland(handle) => handle.invalidate(rect),
        }
    }

    pub fn invalidate_window(&self) {
        match self {
            Handle::Wayland(handle) => handle.invalidate_window(),
        }
    }

    pub fn physical_size(&self) -> PhysicalSize {
        match self {
            Handle::Wayland(handle) => handle.physical_size(),
        }
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        match self {
            Handle::Wayland(handle) => handle.scale_factor(),
        }
    }

    pub fn theme(&self) -> WindowTheme {
        match self {
            Handle::Wayland(handle) => handle.theme(),
        }
    }

    pub fn get_clipboard(&self) -> Result<Option<String>, super::Error> {
        match self {
            Handle::Wayland(handle) => handle.get_clipboard(),
        }
    }

    pub fn set_clipboard(&self, str: &str) -> Result<(), super::Error> {
        match self {
            Handle::Wayland(handle) => handle.set_clipboard(str),
        }
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
