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

#[derive(Default, Clone, PartialEq)]
pub struct FontOptions {
    pub family: FontFamily,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub size: f64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum FontFamily {
    Name(&'static str),
    Serif,
    #[default]
    SansSerif,
}
