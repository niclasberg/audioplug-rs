use super::Color;

pub struct ColorStop {
    pub position: f32,
    pub color: Color
}

pub struct ColorMap {
    pub stops: Vec<ColorStop>
}

impl ColorMap {
    pub fn new(color_positions: impl Into<Vec<ColorStop>>) -> Self {
        Self { stops: color_positions.into() }
    }

    pub fn new_equidistant(colors: &[Color]) -> Self {
        let num_colors = colors.len();
        let stops = colors.into_iter().enumerate().map(|(i, &color)| ColorStop {
            position: (i as f32) / (num_colors as f32),
            color
        }).collect();

        Self {
            stops
        }
    }
}

impl From<(Color, )> for ColorMap {
    fn from(value: (Color, )) -> Self {
        Self::new_equidistant(&[value.0])
    }
}

impl From<(Color, Color,)> for ColorMap {
    fn from(value: (Color, Color,)) -> Self {
        Self::new_equidistant(&[value.0, value.1])
    }
}

impl From<(Color, Color, Color, )> for ColorMap {
    fn from(value: (Color, Color, Color,)) -> Self {
        Self::new_equidistant(&[value.0, value.1, value.2])
    }
}