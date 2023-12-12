use icrate::{AppKit::{NSPasteboard, NSPasteboardTypeString}, Foundation::NSString};

use crate::core::Rectangle;
use super::{view::View, Error};

pub struct HandleRef<'a> {
	view: &'a View
}

impl<'a> HandleRef<'a> {
	pub(crate) fn new(view: &'a View) -> Self {
		Self { view }
	}

	pub fn global_bounds(&self) -> Rectangle {
		unsafe { self.view.bounds().into() }
	}

	pub fn invalidate(&self, rect: Rectangle) {
		unsafe { self.view.setNeedsDisplayInRect(rect.into()) }
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
}