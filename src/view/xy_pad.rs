use crate::core::Point;

pub struct XyPad {
    position: Point
}

pub enum XyPadMessage {
    DragStarted,
    DragEnded,
    ValueChanged
}
