use taffy::prelude::TaffyZero;

use crate::core::Size;

use super::ResolveInto;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Auto,
    /// Length in pixels
    Px(f64),
    /// Length in percent
    Percent(f64),
    /// Percent of viewport height
    Vh(f64),
    /// Percent of viewport width
    Vw(f64),
}

impl Length {
    pub const ZERO: Self = Self::Px(0.0);
    pub const DEFAULT: Self = Self::Auto;

    pub const fn from_px(value: &f64) -> Self {
        Self::Px(*value)
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ResolveInto<taffy::LengthPercentageAuto> for Length {
    fn resolve_into(self, window_size: Size) -> taffy::LengthPercentageAuto {
        match self {
            Length::Auto => taffy::LengthPercentageAuto::auto(),
            Length::Px(val) => taffy::LengthPercentageAuto::length(val as f32),
            Length::Percent(val) => taffy::LengthPercentageAuto::percent((val / 100.0) as _),
            Length::Vh(val) => {
                taffy::LengthPercentageAuto::length((window_size.height * val / 100.0) as _)
            }
            Length::Vw(val) => {
                taffy::LengthPercentageAuto::length((window_size.width * val / 100.0) as _)
            }
        }
    }
}

impl ResolveInto<taffy::LengthPercentage> for Length {
    fn resolve_into(self, window_size: Size) -> taffy::LengthPercentage {
        match self {
            Self::Auto => taffy::LengthPercentage::ZERO,
            Self::Px(val) => taffy::LengthPercentage::length(val as _),
            Self::Percent(val) => taffy::LengthPercentage::percent((val / 100.0) as _),
            Self::Vh(val) => {
                taffy::LengthPercentage::length((window_size.height * val / 100.0) as _)
            }
            Self::Vw(val) => {
                taffy::LengthPercentage::length((window_size.width * val / 100.0) as _)
            }
        }
    }
}

impl ResolveInto<taffy::Dimension> for Length {
    fn resolve_into(self, window_size: Size) -> taffy::Dimension {
        match self {
            Self::Auto => taffy::Dimension::auto(),
            Self::Px(val) => taffy::Dimension::length(val as _),
            Self::Percent(val) => taffy::Dimension::percent((val / 100.0) as _),
            Self::Vh(val) => taffy::Dimension::length((window_size.height * val / 100.0) as _),
            Self::Vw(val) => taffy::Dimension::length((window_size.width * val / 100.0) as _),
        }
    }
}
