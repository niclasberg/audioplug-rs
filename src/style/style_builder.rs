use taffy::Overflow;

use crate::{app::Accessor, core::{Color, Size}};

use super::{Length, UiRect};

pub struct StyleBuilder {
    pub(crate) hidden: Option<Accessor<bool>>,
    pub(crate) padding: Option<Accessor<UiRect>>,
    pub(crate) width: Option<Accessor<Length>>,
    pub(crate) height: Option<Accessor<Length>>,
	pub(crate) min_width: Option<Accessor<Length>>,
	pub(crate) min_height: Option<Accessor<Length>>,
	pub(crate) max_width: Option<Accessor<Length>>,
	pub(crate) max_height: Option<Accessor<Length>>,
	pub(crate) aspect_ratio: Option<Accessor<f64>>,
	pub(crate) border: Option<Accessor<UiRect>>,
	pub(crate) margin: Option<Accessor<UiRect>>,
	pub(crate) inset: Option<Accessor<UiRect>>,
	pub(crate) scrollbar_width: Option<Accessor<f64>>, 
	pub(crate) overflow_x: Option<Accessor<Overflow>>,
	pub(crate) overflow_y: Option<Accessor<Overflow>>,
    pub(crate) background: Option<Accessor<Color>>,
	pub(crate) corner_radius: Option<Accessor<Size>>
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self { 
            hidden: None, 
            padding: None,
            width: None,
            height: None,
            aspect_ratio: None,
            border: None,
            margin: None,
            inset: None,
            scrollbar_width: None,
            overflow_x: None,
            overflow_y: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            background: None,
            corner_radius: None,
        }
    }
}

impl StyleBuilder {
    pub fn hidden(mut self, value: impl Into<Accessor<bool>>) -> Self {
        self.hidden = Some(value.into());
        self
    }

    pub fn padding(mut self, value: impl Into<Accessor<UiRect>>) -> Self {
        self.padding = Some(value.into());
        self
    }

	pub fn margin(mut self, value: impl Into<Accessor<UiRect>>) -> Self {
        self.margin = Some(value.into());
        self
    }

    pub fn height(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.height = Some(value.into());
        self
    }

    pub fn width(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.width = Some(value.into());
        self
    }

    pub fn min_width(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.min_width = Some(value.into());
        self
    }

    pub fn max_width(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.max_width = Some(value.into());
        self
    }

    pub fn min_height(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.min_height = Some(value.into());
        self
    }

    pub fn max_height(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.max_height = Some(value.into());
        self
    }

    pub fn background(mut self, value: impl Into<Accessor<Color>>) -> Self {
        self.background = Some(value.into());
        self
    }

	pub fn corner_radius(mut self, value: impl Into<Accessor<Size>>) -> Self {
		self.corner_radius = Some(value.into());
		self
	}
}