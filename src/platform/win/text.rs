use std::mem::MaybeUninit;

use windows::{
    core::{w, Result, BOOL, HSTRING},
    Win32::Graphics::DirectWrite,
};

use super::{com::direct_write_factory, util};
use crate::core::{Color, FontFamily, FontOptions, FontStyle, FontWeight, Point, Size};

impl From<FontWeight> for DirectWrite::DWRITE_FONT_WEIGHT {
    fn from(val: FontWeight) -> Self {
        match val {
            FontWeight::Bold => DirectWrite::DWRITE_FONT_WEIGHT_BOLD,
            FontWeight::Normal => DirectWrite::DWRITE_FONT_WEIGHT_NORMAL,
        }
    }
}

pub struct NativeTextLayout {
    pub(super) string: String,
    pub(super) text_layout: DirectWrite::IDWriteTextLayout,
    pub(super) color: Color,
}

impl NativeTextLayout {
    pub fn new(string: &str, font: &NativeFont, max_size: Size, color: Color) -> Self {
        let chars: Vec<u16> = string.encode_utf16().collect();

        let family = match font.family {
            FontFamily::Name(name) => HSTRING::from(name),
            FontFamily::Serif => HSTRING::from("Times New Roman"),
            FontFamily::SansSerif => HSTRING::from("Arial"),
        };

        let text_format = unsafe {
            direct_write_factory().CreateTextFormat(
                &family,
                None,
                font.weight.into(),
                DirectWrite::DWRITE_FONT_STYLE_NORMAL,
                DirectWrite::DWRITE_FONT_STRETCH_NORMAL,
                font.size as _,
                w!(""),
            )
        }
        .unwrap();

        let text_layout = unsafe {
            direct_write_factory()
                .CreateTextLayout(
                    chars.as_slice(),
                    &text_format,
                    max_size.width as f32,
                    max_size.height as f32,
                )
                .unwrap()
        };

        Self {
            string: string.to_owned(),
            text_layout,
            color,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn set_max_size(&mut self, size: Size) {
        unsafe {
            let _ = self.text_layout.SetMaxWidth(size.width as f32); //.ok().unwrap();
            let _ = self.text_layout.SetMaxHeight(size.height as f32); //.ok().unwrap();
        }
    }

    pub fn text_index_at_point(&self, point: Point) -> Option<usize> {
        unsafe {
            let mut istrailinghit = BOOL::default();
            let mut isinside = BOOL::default();
            let mut metric = std::mem::MaybeUninit::uninit();
            let result = self.text_layout.HitTestPoint(
                point.x as f32,
                point.y as f32,
                &mut istrailinghit as *mut _,
                &mut isinside as *mut _,
                metric.as_mut_ptr(),
            );

            if result.is_ok() {
                let metric = metric.assume_init();
                let utf16_text_position = if istrailinghit.as_bool() {
                    metric.textPosition + metric.length
                } else {
                    metric.textPosition
                };
                util::count_until_utf16(&self.string, utf16_text_position as usize)
                    .or(Some(self.string.len()))
            } else {
                None
            }
        }
    }

    pub fn point_at_text_index(&self, index: usize) -> Point {
        unsafe {
            let index = index.min(self.string.len());
            assert!(self.string.is_char_boundary(index));
            let index = util::count_utf16(&self.string[..index]);

            let is_trailing_hit = false;
            let mut metric = std::mem::MaybeUninit::uninit();
            let mut pointx = std::mem::MaybeUninit::uninit();
            let mut pointy = std::mem::MaybeUninit::uninit();
            self.text_layout
                .HitTestTextPosition(
                    index as u32,
                    is_trailing_hit,
                    pointx.as_mut_ptr(),
                    pointy.as_mut_ptr(),
                    metric.as_mut_ptr(),
                )
                .expect("Error in HitTestTextPosition");

            let pointx = pointx.assume_init();
            let pointy = pointy.assume_init();

            Point::new(pointx as f64, pointy as f64)
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn measure(&self) -> Size {
        let mut textmetrics = DirectWrite::DWRITE_TEXT_METRICS::default();
        unsafe {
            self.text_layout.GetMetrics(&mut textmetrics as _).unwrap();
        }
        Size::new(textmetrics.width as _, textmetrics.height as _)
    }

    pub fn min_word_width(&self) -> f64 {
        unsafe { self.text_layout.DetermineMinWidth() }.unwrap_or_default() as _
    }
}

pub struct NativeFont {
    family: FontFamily,
    weight: FontWeight,
    style: FontStyle,
    size: f64,
}

impl NativeFont {
    pub fn new(options: &FontOptions) -> Self {
        Self {
            family: options.family,
            weight: options.weight,
            style: options.style,
            size: options.size,
        }
    }
}

pub struct FontCollection {
    collection: DirectWrite::IDWriteFontCollection,
}

impl FontCollection {
    pub fn new() -> Result<Self> {
        let collection = unsafe {
            let mut collection = MaybeUninit::uninit();
            direct_write_factory().GetSystemFontCollection(collection.as_mut_ptr(), true)?;
            collection.assume_init()
        };
        let collection = collection.unwrap();

        Ok(Self { collection })
    }
}
