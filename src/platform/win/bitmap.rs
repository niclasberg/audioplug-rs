use std::mem::MaybeUninit;
use std::path::Path;

use windows::Win32::{Foundation::GENERIC_READ, Graphics::Imaging};
use windows::core::{HSTRING, Result};

use crate::core::Size;

use super::com::wic_factory;

pub struct Bitmap {
    converter: Imaging::IWICFormatConverter,
}

impl Bitmap {
    pub fn from_file(path: &Path) -> Result<Self> {
        let decoder = unsafe {
            wic_factory().CreateDecoderFromFilename(
                &HSTRING::from(path),
                None,
                GENERIC_READ,
                Imaging::WICDecodeMetadataCacheOnLoad,
            )
        }?;
        let converter = unsafe { wic_factory().CreateFormatConverter() }?;

        let frame = unsafe { decoder.GetFrame(0) }?;
        unsafe {
            converter.Initialize(
                &frame,
                &Imaging::GUID_WICPixelFormat32bppPBGRA,
                Imaging::WICBitmapDitherTypeNone,
                None,
                0.0,
                Imaging::WICBitmapPaletteTypeCustom,
            )
        }?;

        Ok(Self { converter })
    }

    pub fn size(&self) -> Size {
        let (width, height) = unsafe {
            let mut width = MaybeUninit::uninit();
            let mut height = MaybeUninit::uninit();
            if let Ok(()) = self
                .converter
                .GetSize(width.as_mut_ptr(), height.as_mut_ptr())
            {
                (width.assume_init(), height.assume_init())
            } else {
                (0, 0)
            }
        };
        [width, height].into()
    }
}
