use crate::{text::FontWeight, core::Size};

pub struct TextLayout(
    
);

impl TextLayout {
    pub fn new(
        string: &str, 
        font_family: &str, 
        font_weight: FontWeight,
        font_size: f32,
        max_size: Size
    ) -> Self {
        todo!()
    }

    pub fn set_max_size(&mut self, size: Size) {
        todo!()
    }

    pub fn measure(&self) -> Size {
        todo!()
    }
}