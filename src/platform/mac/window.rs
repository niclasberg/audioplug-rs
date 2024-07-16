use objc2_foundation::{NSPoint, NSRect, NSSize, MainThreadMarker};
use objc2_app_kit::{NSWindow, NSBackingStoreType, NSWindowStyleMask};
use objc2::rc::Id;
use objc2::ClassType;
use crate::core::Rectangle;
use crate::platform::WindowHandler;

use super::Error;
use super::view::View;

pub struct Window {
	window: Id<NSWindow>,
	view: Id<View>
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

		Ok(Self { window, view })
	}

	pub fn set_size(&self, size: Rectangle<i32>) -> Result<(), Error> {
		todo!()
	}
}