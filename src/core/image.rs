use std::rc::Rc;

use crate::core::Size;

pub struct Image(Rc<ImageInner>);

impl Image {
    pub fn size(&self) -> Size {
        Size::ZERO
    }
}

struct ImageInner {}
