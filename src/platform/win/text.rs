use windows::{Win32::Graphics::DirectWrite, core::HSTRING, w};

use crate::{core::Size, text::FontWeight};
use super::com::direct_write_factory;

impl Into<DirectWrite::DWRITE_FONT_WEIGHT> for FontWeight {
    fn into(self) -> DirectWrite::DWRITE_FONT_WEIGHT {
        match self {
            FontWeight::Bold => DirectWrite::DWRITE_FONT_WEIGHT_BOLD,
            FontWeight::Normal => DirectWrite::DWRITE_FONT_WEIGHT_NORMAL,
        }
    }
}

pub struct TextLayout(
    pub(super) DirectWrite::IDWriteTextLayout
);

impl TextLayout {
    pub fn new(
        string: &str, 
        font_family: &str, 
        font_weight: FontWeight,
        font_size: f32,
        max_size: Size
    ) -> Self {
        let chars: Vec<u16> = string.encode_utf16().collect();

        let text_format = unsafe {
            direct_write_factory().CreateTextFormat(
                &HSTRING::from(font_family),
                None, 
                font_weight.into(), 
                DirectWrite::DWRITE_FONT_STYLE_NORMAL, 
                DirectWrite::DWRITE_FONT_STRETCH_NORMAL, 
                font_size, 
                w!(""))
        }.unwrap();

        let text_layout = unsafe { 
            direct_write_factory().CreateTextLayout(
                chars.as_slice(), 
                &text_format, 
                max_size.width as f32, 
                max_size.height as f32
            ).unwrap() 
        };
        Self(text_layout)
    }

    pub fn set_max_size(&mut self, size: Size) {
        unsafe {
            self.0.SetMaxWidth(size.width as f32).ok().unwrap();
            self.0.SetMaxHeight(size.height as f32).ok().unwrap();
        }
    }

    pub fn measure(&self) -> Size {
        let mut textmetrics = DirectWrite::DWRITE_TEXT_METRICS::default();
        unsafe { self.0.GetMetrics(&mut textmetrics as _).unwrap(); }
        Size::new(textmetrics.width as _, textmetrics.height as _)
    }
}