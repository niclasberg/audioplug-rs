use std::{ffi::c_void, ptr::NonNull};

use dispatch2::MainThreadBound;
use objc2::{
    MainThreadMarker,
    rc::{Retained, Weak},
};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;
use raw_window_handle::{
    AppKitWindowHandle, DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
    RawWindowHandle, WindowHandle,
};

use super::{Error, view::View};
use crate::core::{Rectangle, Size, WindowTheme};

pub struct Handle {
    view: MainThreadBound<Weak<View>>,
}

impl Handle {
    pub(crate) fn new(view: MainThreadBound<Weak<View>>) -> Self {
        Self { view }
    }

    fn view_ref(&self) -> Option<Retained<View>> {
        let mtm = MainThreadMarker::new().expect("Must be called on the main thread");
        self.view.get(mtm).load()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.view_ref()
            .map(|view| view.bounds().into())
            .unwrap_or_default()
    }

    pub fn theme(&self) -> WindowTheme {
        WindowTheme::Dark
    }

    pub fn invalidate_window(&self) {
        if let Some(view) = self.view_ref() {
            unsafe { view.setNeedsDisplay(true) }
        }
    }

    pub fn invalidate(&self, rect: Rectangle) {
        if let Some(view) = self.view_ref() {
            unsafe { view.setNeedsDisplayInRect(rect.into()) }
        }
    }

    pub fn set_clipboard(&self, string: &str) -> Result<(), Error> {
        unsafe {
            let string = NSString::from_str(string);
            let pasteboard = NSPasteboard::generalPasteboard();
            let _ = pasteboard.clearContents();

            pasteboard.setString_forType(string.as_ref(), &NSPasteboardTypeString);

            Ok(())
        }
    }

    pub fn get_clipboard(&self) -> Result<Option<String>, Error> {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            let string = pasteboard.stringForType(&NSPasteboardTypeString);
            Ok(string.map(|str| str.to_string()))
        }
    }

    pub fn physical_size(&self) -> Size<u32> {
        todo!()
    }
}

impl HasWindowHandle for Handle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if let Some(mtm) = MainThreadMarker::new() {
            let view_ptr = self
                .view
                .get(mtm)
                .load()
                .as_ref()
                .map(Retained::as_ptr)
                .ok_or(HandleError::Unavailable)?;

            let view_ptr =
                NonNull::new(view_ptr as *mut c_void).expect("View pointer should never be null");
            let handle = AppKitWindowHandle::new(view_ptr);
            let handle = RawWindowHandle::AppKit(handle);
            Ok(unsafe { WindowHandle::borrow_raw(handle) })
        } else {
            Err(HandleError::Unavailable)
        }
    }
}

impl HasDisplayHandle for Handle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::appkit())
    }
}
