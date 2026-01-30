use raw_window_handle::{WaylandWindowHandle, XcbWindowHandle};

use crate::{
    core::{PhysicalRect, Rect, ScaleFactor},
    platform::{
        Application, WindowHandler,
        linux::{wayland::window::WaylandWindow, x11::X11Window},
    },
};

pub enum Window {
    Wayland(WaylandWindow),
    X11(X11Window),
}

impl Window {
    pub fn open(
        app: &mut Application,
        handler: Box<dyn WindowHandler>,
    ) -> Result<Self, super::Error> {
        match app {
            Application::Wayland(app) => WaylandWindow::open(app, handler).map(Self::Wayland),
            Application::X11(app) => X11Window::open(app, handler).map(Self::X11),
        }
    }

    pub fn attach_wayland(
        handle: WaylandWindowHandle,
        handler: Box<dyn WindowHandler>,
    ) -> Result<Self, super::Error> {
        WaylandWindow::attach(handle, handler).map(Self::Wayland)
    }

    pub fn attach_xcb(
        _handle: XcbWindowHandle,
        _handler: Box<dyn WindowHandler>,
    ) -> Result<Self, super::Error> {
        todo!()
    }

    pub fn set_scale_factor(&self, scale_factor: ScaleFactor) {
        match self {
            Self::Wayland(wayland_window) => wayland_window.set_scale_factor(scale_factor),
            Self::X11(_) => todo!(),
        }
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        ScaleFactor::default()
    }

    pub fn set_physical_size(&self, size: PhysicalRect) -> Result<(), super::Error> {
        match self {
            Window::Wayland(wayland_window) => wayland_window.set_physical_size(size),
            Self::X11(_) => todo!(),
        }
    }

    pub fn set_logical_size(&self, rect: Rect) -> Result<(), super::Error> {
        match self {
            Window::Wayland(wayland_window) => wayland_window.set_logical_size(rect),
            Self::X11(_) => todo!(),
        }
    }
}
