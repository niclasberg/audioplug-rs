use std::path::Path;
use crate::core::Size;

use objc2::{rc::Retained, AllocAnyThread};
use objc2_foundation::NSString;
use objc2_app_kit::NSImage;

use super::Error;

#[derive(Debug, Clone)]
pub struct ImageSource(pub(super) Retained<NSImage>);

impl ImageSource {
	pub fn from_file(path: &Path) -> Result<Self, Error> {
		let path_str = NSString::from_str(path.to_str().unwrap());
		let ns_image = unsafe {
			NSImage::initWithContentsOfFile(NSImage::alloc(), &path_str)
		};
		ns_image.map(|ns_image| Self(ns_image)).ok_or(Error)
	}

	pub fn size(&self) -> Size {
		let representations = unsafe { self.0.representations() };
		representations.firstObject()
			.map(|representation| {
				let size = unsafe { representation.size() };
				Size::new(size.width, size.height)
			})
			.unwrap_or_default()
	}
}