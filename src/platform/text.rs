use crate::core::{Color, FontFamily, FontOptions, FontWeight, Point, Size};
use crate::platform::{NativeFont, NativeTextLayout};

pub struct Font {
    native: NativeFont,
}

impl Font {
    pub fn from_family_and_size(family: &str, size: f64) -> Self {
        let native = NativeFont::new(&FontOptions {
            family: FontFamily::Name(family.to_string()),
            size,
            ..Default::default()
        });
        Self { native }
    }

    pub fn new(options: &FontOptions) -> Self {
        let native = NativeFont::new(options);
        Self { native }
    }

    pub fn family_name(&self) -> String {
        self.native.family_name()
    }
}

pub struct TextLayout(pub(crate) NativeTextLayout);

impl TextLayout {
    pub fn new(str: &str, color: Color, max_size: Size) -> Self {
        Self(NativeTextLayout::new(
            str,
            &Font::new(&FontOptions::default()).native,
            max_size,
            color,
        ))
    }

    pub fn text_index_at_point(&self, point: impl Into<Point>) -> Option<usize> {
        self.0.text_index_at_point(point.into())
    }

    pub fn point_at_text_index(&self, index: usize) -> Point {
        self.0.point_at_text_index(index)
    }

    pub fn measure(&self) -> Size {
        self.0.measure()
    }

    pub fn min_word_width(&self) -> f64 {
        self.0.min_word_width()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn set_color(&mut self, _color: Color) {}

    pub fn set_max_size(&mut self, size: Size) {
        self.0.set_max_size(size)
    }
}
