use crate::core::{Color, FontFamily, FontOptions, FontStyle, FontWeight, Point, Size, Zero};


pub struct NativeTextLayout {
    pub(super) string: String,
    pub(super) color: Color,
}

impl NativeTextLayout {
    pub fn new(string: &str, _font: &NativeFont, _max_size: Size, color: Color) -> Self {

        Self {
            string: string.to_owned(),
            color,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn set_max_size(&mut self, _size: Size) {
        
    }

    pub fn text_index_at_point(&self, _point: Point) -> Option<usize> {
        None
    }

    pub fn point_at_text_index(&self, _index: usize) -> Point {
        Point::ZERO
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn measure(&self) -> Size {
        Size::ZERO
    }

    pub fn min_word_width(&self) -> f64 {
        0.0
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