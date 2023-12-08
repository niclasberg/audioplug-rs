use crate::{platform, core::{Size, Point}, core::Color};

pub enum FontWeight {
    Normal,
    Bold,
}

pub struct TextLayout(pub(crate) platform::TextLayout);

impl TextLayout {
    pub fn new(str: &str, color: Color, max_size: Size) -> Self {
        Self(
            platform::TextLayout::new(
                str, 
                "arial", 
                FontWeight::Normal, 
                12.0, 
                max_size, 
                color)
        )
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

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn set_max_size(&mut self, size: Size) {
        self.0.set_max_size(size)
    }
}