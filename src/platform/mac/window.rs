use std::ffi::c_void;
use std::ptr::NonNull;

use crate::core::Rectangle;
use crate::platform::WindowHandler;
use objc2::rc::{Retained, Weak};
use objc2_app_kit::{NSBackingStoreType, NSView, NSWindow, NSWindowStyleMask};
use objc2_core_foundation::CGSize;
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize};
use raw_window_handle::{AppKitWindowHandle, HasWindowHandle, RawWindowHandle};

use super::view::View;
use super::Error;

pub enum Window {
    OSWindow(Retained<NSWindow>, Retained<View>),
    AttachedToView(Weak<View>),
}

impl Window {
    pub(crate) fn open(widget: impl WindowHandler + 'static) -> Result<Self, Error> {
        let mtm = MainThreadMarker::new().unwrap();
        let content_rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(1024., 768.));
        let window = {
            let this = mtm.alloc();
            let backing_store_type = NSBackingStoreType::Buffered;
            let style = NSWindowStyleMask::Closable
                | NSWindowStyleMask::Resizable
                | NSWindowStyleMask::Titled;
            let flag = false;

            unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    this,
                    content_rect,
                    style,
                    backing_store_type,
                    flag,
                )
            }
        };

        let view = View::new(mtm, widget, Some(content_rect));

        window.makeKeyAndOrderFront(None);
        window.setContentView(Some(&*view));

        Ok(Self::OSWindow(window, view))
    }

    pub fn attach(
        handle: AppKitWindowHandle,
        widget: impl WindowHandler + 'static,
    ) -> Result<Self, Error> {
        let mtm = MainThreadMarker::new().unwrap();
        let parent = unsafe { &*(handle.ns_view.as_ptr() as *mut NSView) };
        let view = View::new(mtm, widget, None);
        unsafe {
            parent.addSubview(&view);
            view.setNeedsDisplay(true);
        };

        Ok(Self::AttachedToView(Weak::from_retained(&view)))
    }

    pub fn set_size(&self, size: Rectangle<i32>) -> Result<(), Error> {
        let size = CGSize::new(size.width() as f64, size.height() as f64);
        match self {
            Window::OSWindow(_, _) => Ok(()),
            Window::AttachedToView(view) => {
                if let Some(view) = view.load() {
                    unsafe { view.setFrameSize(size) };
                    Ok(())
                } else {
                    Err(Error)
                }
            }
        }
    }

    pub fn set_scale_factor(&self, _scale_factor: f32) {}

    pub fn size(&self) -> Result<Rectangle<i32>, Error> {
        let frame = match self {
            Window::OSWindow(view, _) => Ok(view.frame()),
            Window::AttachedToView(view) => {
                if let Some(view) = view.load() {
                    Ok(view.frame())
                } else {
                    Err(Error)
                }
            }
        }?;
        todo!()
    }
}

impl HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let view = match self {
            Window::OSWindow(_, view) => Ok(view.clone()),
            Window::AttachedToView(view) => view
                .load()
                .as_ref()
                .map(Retained::clone)
                .ok_or(raw_window_handle::HandleError::Unavailable),
        }?;

        let view_ptr = unsafe { NonNull::new_unchecked(Retained::into_raw(view) as *mut c_void) };
        let handle = AppKitWindowHandle::new(view_ptr);
        let handle = RawWindowHandle::AppKit(handle);
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(handle) })
    }
}
