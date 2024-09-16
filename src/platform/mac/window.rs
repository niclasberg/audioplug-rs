use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_foundation::{CGSize, MainThreadMarker, NSPoint, NSRect, NSSize};
use objc2_app_kit::{NSBackingStoreType, NSView, NSWindow, NSWindowStyleMask};
use objc2::rc::{Retained, Weak};
use raw_window_handle::{AppKitWindowHandle, HasWindowHandle, RawWindowHandle};
use crate::core::{Rectangle, Size};
use crate::platform::WindowHandler;

use super::Error;
use super::view::View;

pub enum Window {
	OSWindow(Retained<NSWindow>, Retained<View>),
	AttachedToView(Weak<View>)
}

impl Window {
	pub(crate) fn open(widget: impl WindowHandler + 'static) -> Result<Self, Error> {
		let mtm = MainThreadMarker::new().unwrap();
		let window = {
			let this = mtm.alloc();
			let backing_store_type = NSBackingStoreType::NSBackingStoreBuffered;
			let content_rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(1024., 768.));
			let style = NSWindowStyleMask::Closable | NSWindowStyleMask::Resizable | NSWindowStyleMask::Titled;
			let flag = false;
	
			unsafe {
				NSWindow::initWithContentRect_styleMask_backing_defer(
					this, 
					content_rect, 
					style, 
					backing_store_type, 
					flag)
			}
		};

		let view = View::new(mtm, widget);

		window.makeKeyAndOrderFront(None);
		window.setContentView(Some(&*view));

		Ok(Self::OSWindow(window, view))
	}

	pub fn attach(handle: AppKitWindowHandle, widget: impl WindowHandler + 'static) -> Result<Self, Error> {
		let mtm = MainThreadMarker::new().unwrap();
		let parent = unsafe { &*(handle.ns_view.as_ptr() as *mut NSView) };
		let view = View::new(mtm, widget);
		unsafe { 
			parent.addSubview(&view);
			view.setFrame(parent.frame());
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
			},
		}
	}
}

impl HasWindowHandle for Window {
	fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
		let view = match self {
			Window::OSWindow(_, view) => Ok(view.clone()),
			Window::AttachedToView(view) => view.load().as_ref().map(Retained::clone).ok_or(raw_window_handle::HandleError::Unavailable),
		}?;

		let view_ptr = unsafe { NonNull::new_unchecked(Retained::into_raw(view) as *mut c_void) };
		let handle = AppKitWindowHandle::new(view_ptr);
		let handle = RawWindowHandle::AppKit(handle);
		Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(handle) })
	}
}