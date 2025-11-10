use std::{ffi::c_void, ptr::NonNull};

use dispatch2::MainThreadBound;
use objc2::{
    MainThreadMarker,
    rc::{Retained, Weak},
};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, HandleError, RawDisplayHandle, RawWindowHandle,
};

use super::{Error, view::View};
use crate::core::{PhysicalSize, Rect, WindowTheme};

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

    pub fn global_bounds(&self) -> Rect {
        self.view_ref()
            .map(|view| view.bounds().into())
            .unwrap_or_default()
    }

    pub fn theme(&self) -> WindowTheme {
        WindowTheme::Dark
    }

    pub fn invalidate_window(&self) {
        if let Some(view) = self.view_ref() {
            view.setNeedsDisplay(true)
        }
    }

    pub fn invalidate(&self, rect: Rect) {
        if let Some(view) = self.view_ref() {
            view.setNeedsDisplayInRect(rect.into())
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

    pub fn physical_size(&self) -> PhysicalSize {
        self.view_ref()
            .map(|view| view.physical_size())
            .unwrap_or_default()
    }

    pub fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
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
            Ok(handle)
        } else {
            Err(HandleError::Unavailable)
        }
    }

    pub fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        Ok(RawDisplayHandle::AppKit(AppKitDisplayHandle::new()))
    }
}
