use objc2::rc::Weak;
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;

use crate::core::Rectangle;
use super::{view::View, Error};

pub struct Handle {
	view: Weak<View>
}

impl Handle {
	pub(crate) fn new(view: Weak<View>) -> Self {
		Self { view }
	}

	pub fn global_bounds(&self) -> Rectangle {
		self.view.load().map(|view| view.bounds().into()).unwrap_or_default()
	}

	pub fn invalidate_window(&self) {
		if let Some(view) = self.view.load() {
			unsafe { view.setNeedsDisplay(true) }
		}
	}

	pub fn invalidate(&self, rect: Rectangle) {
		if let Some(view) = self.view.load() {
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
}