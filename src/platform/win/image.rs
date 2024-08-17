use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::path::Path;

use windows::core::{Result, HSTRING};
use windows::Win32::{Foundation::GENERIC_READ, Graphics::{Imaging, Direct2D}};

use crate::core::{Rectangle, Size};

use super::com::wic_factory;
use super::renderer::RendererGeneration;

struct CachedD2D1Bitmap {
    generation: RendererGeneration,
    bitmap: Direct2D::ID2D1Bitmap
}

pub struct ImageSource {
    converter: Imaging::IWICFormatConverter,
    cached_bitmap: RefCell<Option<CachedD2D1Bitmap>>
}

impl ImageSource {
    pub fn from_file(path: &Path) -> Result<Self> {
        let decoder = unsafe { wic_factory().CreateDecoderFromFilename(&HSTRING::from(path), None, GENERIC_READ, Imaging::WICDecodeMetadataCacheOnLoad) }?;
        let converter = unsafe { wic_factory().CreateFormatConverter() }?;
        
        let frame = unsafe { decoder.GetFrame(0) }?;
        unsafe { 
            converter.Initialize(
                &frame, 
                &Imaging::GUID_WICPixelFormat32bppPBGRA, 
                Imaging::WICBitmapDitherTypeNone, 
                None, 
                0.0, 
                Imaging::WICBitmapPaletteTypeCustom)
        }?;

        Ok(Self {
            converter,
            cached_bitmap: RefCell::new(None)
        })
    }

    pub fn size(&self) -> Size {
        let (width, height) = unsafe { 
            let mut width = MaybeUninit::uninit();
            let mut height = MaybeUninit::uninit();
            if let Ok(()) = self.converter.GetSize(width.as_mut_ptr(), height.as_mut_ptr()) {
                (width.assume_init(), height.assume_init())
            } else {
                (0, 0)
            }
        };
        [width, height].into()
    }

    pub(super) fn draw(&self, render_target: &Direct2D::ID2D1HwndRenderTarget, generation: RendererGeneration, rect: Direct2D::Common::D2D_RECT_F) {
        if !self.cached_bitmap.borrow().as_ref().is_some_and(|x| x.generation == generation) {
            let mut cached_bitmap = self.cached_bitmap.borrow_mut();
            let bitmap = unsafe { render_target.CreateBitmapFromWicBitmap(&self.converter, None) }.unwrap();
            *cached_bitmap = Some(CachedD2D1Bitmap {
                generation,
                bitmap,
            });    
        }
        
        if let Some(cached_bitmap) = self.cached_bitmap.borrow().as_ref() {
            let opacity = 1.0;
            let interpolation_mode = Direct2D::D2D1_BITMAP_INTERPOLATION_MODE_LINEAR; 
            unsafe {
                render_target.DrawBitmap(
                    &cached_bitmap.bitmap, 
                    Some(&rect as *const _), 
                    opacity, 
                    interpolation_mode, 
                    None)
            }
        }
    }
}