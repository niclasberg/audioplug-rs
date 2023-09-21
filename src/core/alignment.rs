use super::{Size, Point};

pub enum Alignment {
    TopLeading,
    Top,
    TopTrailing,
    Leading,
    Center,
    Trailing,
    BottomLeading,
    Bottom,
    BottomTrailing,
}

impl Alignment {
    pub fn compute_offset(&self, size: Size, frame_size: Size) -> Point {
        let x = match self {
            Self::TopLeading | Self::Leading | Self::BottomLeading => 0.0,
            Self::Top | Self::Center | Self::Bottom => (frame_size.width - size.width) / 2.0,
            Self::TopTrailing | Self::Trailing | Self::BottomTrailing => frame_size.width - size.width,
        };
        let y = match self {
            Self::TopLeading | Self::Top | Self::TopTrailing => 0.0,
            Self::Leading | Self::Center | Self::Trailing => (frame_size.height - size.height) / 2.0,
            Self::BottomLeading | Self::Bottom | Self::BottomTrailing => frame_size.height - size.height
        };
        Point::new(x, y)
    }
}