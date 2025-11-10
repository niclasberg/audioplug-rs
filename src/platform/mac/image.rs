use crate::core::Size;
use std::path::Path;

use objc2::{AllocAnyThread, rc::Retained};
use objc2_app_kit::NSImage;
use objc2_foundation::NSString;

use super::Error;

#[derive(Debug, Clone)]
pub struct Bitmap(pub(super) Retained<NSImage>);

impl Bitmap {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let path_str = NSString::from_str(path.to_str().unwrap());
        let ns_image = NSImage::initWithContentsOfFile(NSImage::alloc(), &path_str);
        ns_image.map(|ns_image| Self(ns_image)).ok_or(Error)
    }

    pub fn size(&self) -> Size {
        let representations = self.0.representations();
        representations
            .firstObject()
            .map(|representation| {
                let size = representation.size();
                Size::new(size.width, size.height)
            })
            .unwrap_or_default()
    }
}
