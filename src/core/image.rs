use std::path::Path;

use crate::core::Size;

pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Box<[u8]>,
}

impl ImageData {
    pub fn from_file(path: impl AsRef<Path>) -> image::ImageResult<Self> {
        let image = image::open(path)?.into_rgba8();
        Ok(Self {
            width: image.width(),
            height: image.height(),
            data: image.into_raw().into_boxed_slice(),
        })
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width as _,
            height: self.height as _,
        }
    }
}
