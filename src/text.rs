use crate::{platform, core::Size};

pub enum FontWeight {
    Normal,
    Bold,
}

pub struct TextLayout(pub(crate) platform::TextLayout);

impl TextLayout {
    pub fn new(str: &str, max_size: Size) -> Self {
        Self(
            platform::TextLayout::new(
                str, 
                "verdana", 
                FontWeight::Normal, 
                18.0, 
                max_size)
        )
    }

    pub fn measure(&self) -> Size {
        self.0.measure()
    }

    pub fn set_max_size(&mut self, size: Size) {
        self.0.set_max_size(size)
    }
}