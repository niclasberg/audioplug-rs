use icrate::Foundation::{NSPoint, NSRect, NSSize};
use icrate::AppKit::{NSWindow, NSBackingStoreBuffered, NSWindowStyleMaskClosable, NSWindowStyleMaskResizable, NSWindowStyleMaskTitled};
use objc2::rc::Id;
use objc2::ClassType;

use crate::widget::Widget;
use super::view::View;

pub struct Window {
	window: Id<NSWindow>,
	view: Id<View>
}

impl Window {
	pub(crate) fn new(widget: impl Widget + 'static) -> Result<Self, ()> {
		let window = {
			let this = NSWindow::alloc();
			let backing_store_type = NSBackingStoreBuffered;
			let content_rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(1024., 768.));
			let style = NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskTitled;
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

		let view = View::new();

		unsafe {
			window.makeKeyAndOrderFront(None);
			window.setContentView(Some(&*view));
		}

		Ok(Self { window, view })
	}
}