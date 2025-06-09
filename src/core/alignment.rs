use super::{Size, Vec2};

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
    pub fn compute_offset_x(&self, width: f64, frame_width: f64) -> f64 {
        match self {
            Self::TopLeading | Self::Leading | Self::BottomLeading => 0.0,
            Self::Top | Self::Center | Self::Bottom => (frame_width - width) / 2.0,
            Self::TopTrailing | Self::Trailing | Self::BottomTrailing => frame_width - width,
        }
    }

    pub fn compute_offset_y(&self, height: f64, frame_height: f64) -> f64 {
        match self {
            Self::TopLeading | Self::Top | Self::TopTrailing => 0.0,
            Self::Leading | Self::Center | Self::Trailing => (frame_height - height) / 2.0,
            Self::BottomLeading | Self::Bottom | Self::BottomTrailing => frame_height - height,
        }
    }

    pub fn compute_offset(&self, size: Size, frame_size: Size) -> Vec2 {
        let x = self.compute_offset_x(size.width, frame_size.width);
        let y = self.compute_offset_y(size.height, frame_size.height);
        Vec2::new(x, y)
    }
}
