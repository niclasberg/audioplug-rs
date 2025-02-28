use super::{Color, Point, UnitPoint};

#[derive(Debug, Clone, Copy)]
pub struct ColorStop {
    pub position: f32,
    pub color: Color
}

#[derive(Debug, Clone)]
pub struct ColorMap {
    pub stops: Vec<ColorStop>
}

impl ColorMap {
    pub fn new(color_positions: impl Into<Vec<ColorStop>>) -> Self {
        Self { stops: color_positions.into() }
    }

    pub fn new_equidistant(colors: &[Color]) -> Self {
        assert!(colors.len() > 0);
        let denum = if colors.len() == 1 { 1.0 } else { (colors.len() - 1) as f32 };
        let stops = colors.into_iter().enumerate().map(|(i, &color)| ColorStop {
            position: (i as f32) / denum,
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
