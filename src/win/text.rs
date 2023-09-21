use windows::Win32::Graphics::DirectWrite;

use crate::core::Size;
use super::com::direct_write_factory;

pub(crate) struct TextFormat(
    pub(super) DirectWrite::IDWriteTextFormat
);

impl TextFormat {
    pub fn new() -> Self {
        /*let text_format = direct_write_factory().CreateTextFormat(
            fontfamilyname, 
            fontcollection, 
            fontweight, 
            fontstyle, 
            fontstretch, 
            fontsize, 
            localename);*/
        todo!()
    }
}

pub(crate) struct TextLayout(
    pub(super) DirectWrite::IDWriteTextLayout
);

impl TextLayout {
    pub fn new(string: &str, format: &TextFormat, max_size: Size) -> Self {
        let chars: Vec<u16> = string.encode_utf16().collect();
        let text_layout = unsafe { 
            direct_write_factory().CreateTextLayout(
                chars.as_slice(), 
                &format.0, 
                max_size.width as f32, 
                max_size.height as f32
            ).unwrap() 
        };
        Self(text_layout)
    }

    pub fn measure(&self) -> Size {
        let mut textmetrics = DirectWrite::DWRITE_TEXT_METRICS::default();
        unsafe { self.0.GetMetrics(&mut textmetrics as _).unwrap(); }
        Size::new(textmetrics.width as _, textmetrics.height as _)
    }
}