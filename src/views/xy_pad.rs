use crate::{View, core::Point};

pub struct XyPad {
    position: Point
}

pub enum XyPadMessage {
    DragStarted,
    DragEnded,
    ValueChanged
}
