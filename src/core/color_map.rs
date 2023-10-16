use super::Color;

pub struct ColorPosition {
    pub position: f32,
    pub color: Color
}

pub struct ColorMap {
    color_positions: Vec<ColorPosition>
}

impl ColorMap {
    pub fn new(color_positions: impl Into<Vec<ColorPosition>>) -> Self {
        Self { color_positions: color_positions.into() }
    }
}