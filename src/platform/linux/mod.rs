mod application;
mod audio;
mod executor;
mod text;
mod wayland;
mod window;
mod x11;

use std::fmt::Display;

pub use application::Application;
pub use audio::{AudioHost, Device};
pub use executor::Executor;
use raw_window_handle::{HandleError, RawDisplayHandle, RawWindowHandle};
pub use text::{NativeFont, NativeTextLayout};
pub use window::Window;

use crate::{
    core::{PhysicalSize, Rect, ScaleFactor, Size, WindowTheme, Zero},
    platform::linux::{wayland::handle::WaylandHandle, x11::X11Handle},
};

#[derive(Debug, Clone)]
pub struct Error {
    reason: String,
}

impl Error {
    pub fn from_reason(reason: impl ToString) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reason)
    }
}

impl std::error::Error for Error {}

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
    Wayland(WaylandHandle),
    X11(X11Handle),
}

impl Handle {
    pub fn global_bounds(&self) -> Rect {
        match self {
            Handle::Wayland(handle) => handle.global_bounds(),
            Handle::X11(handle) => handle.global_bounds(),
        }
    }

    pub fn invalidate(&self, rect: Rect) {
        match self {
            Handle::Wayland(handle) => handle.invalidate(rect),
            Handle::X11(handle) => handle.invalidate(rect),
        }
    }

    pub fn invalidate_window(&self) {
        match self {
            Handle::Wayland(handle) => handle.invalidate_window(),
            Handle::X11(handle) => handle.invalidate_window(),
        }
    }

    pub fn physical_size(&self) -> PhysicalSize {
        match self {
            Handle::Wayland(handle) => handle.physical_size(),
            Handle::X11(handle) => handle.physical_size(),
        }
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        match self {
            Handle::Wayland(handle) => handle.scale_factor(),
            Handle::X11(handle) => handle.scale_factor(),
        }
    }

    pub fn theme(&self) -> WindowTheme {
        match self {
            Handle::Wayland(handle) => handle.theme(),
            Handle::X11(handle) => handle.theme(),
        }
    }

    pub fn get_clipboard(&self) -> Result<Option<String>, super::Error> {
        match self {
            Handle::Wayland(handle) => handle.get_clipboard(),
            Handle::X11(handle) => handle.get_clipboard(),
        }
    }

    pub fn set_clipboard(&self, str: &str) -> Result<(), super::Error> {
        match self {
            Handle::Wayland(handle) => handle.set_clipboard(str),
            Handle::X11(handle) => handle.set_clipboard(str),
        }
    }

    pub fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
        match self {
            Handle::Wayland(handle) => handle.raw_window_handle(),
            Handle::X11(handle) => handle.raw_window_handle(),
        }
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        match self {
            Handle::Wayland(handle) => handle.raw_display_handle(),
            Handle::X11(handle) => handle.raw_display_handle(),
        }
    }
}
