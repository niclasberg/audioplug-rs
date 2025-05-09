use crate::app::Accessor;

pub enum ImageFilter {
    GaussianBlur { radius: Accessor<f64> },
    Opacity { value: Accessor<f64> },
}

impl ImageFilter {
    pub fn gaussian_blur(radius: impl Into<Accessor<f64>>) -> Self {
        Self::GaussianBlur {
            radius: radius.into(),
        }
    }

    pub fn opacity(value: impl Into<Accessor<f64>>) -> Self {
        Self::Opacity {
            value: value.into(),
        }
    }
}
