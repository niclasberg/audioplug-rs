#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum FontFamily {
    Name(&'static str),
    Serif,
    #[default]
    SansSerif,
}

#[derive(Clone, Copy, PartialEq)]
pub struct FontOptions {
    pub family: FontFamily,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub size: f64,
}

impl Default for FontOptions {
    fn default() -> Self {
        Self {
            family: Default::default(),
            weight: Default::default(),
            style: Default::default(),
            size: 12.0,
        }
    }
}

pub struct TextLayout {}
