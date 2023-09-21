#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0
    };

    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0, 
        b: 1.0,
        a: 1.0
    };

    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0
    };

    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0
    };

    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0
    };

    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self{
        Self {r, g, b, a}
    }

    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self{
        Self::from_rgba(r, g, b, 1.0)
    }
}