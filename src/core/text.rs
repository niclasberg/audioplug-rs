use std::rc::Rc;

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

pub(crate) struct FontFamilyInner {
    pub family: String,
    pub variations: Vec<Font>,
}

pub struct FontFamily(Rc<FontFamilyInner>);

impl FontFamily {
    pub fn name(&self) -> &str {
        &self.0.family
    }

    pub fn fonts(&self) -> &[Font] {
        &self.0.variations
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct FontInner {
    pub name: String,
    pub weight: FontWeight,
    pub style: FontStyle,
}

#[derive(Clone, PartialEq)]
pub struct Font(Rc<FontInner>);

impl Font {
    pub fn new(name: &str, weight: FontWeight, style: FontStyle) -> Self {
        let inner = FontInner {
            name: name.to_string(),
            weight,
            style,
        };
        Self(Rc::new(inner))
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn weight(&self) -> FontWeight {
        self.0.weight
    }

    pub fn style(&self) -> FontStyle {
        self.0.style
    }
}
