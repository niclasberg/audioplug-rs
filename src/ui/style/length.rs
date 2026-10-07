use taffy::prelude::TaffyZero;

use crate::core::{Point, Rect, Size};

use super::ResolveInto;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Auto,
    /// Length in pixels
    Px(f32),
    /// Length in percent
    Percent(f32),
    /// Percent of viewport height
    Vh(f32),
    /// Percent of viewport width
    Vw(f32),
}

impl Length {
    pub const ZERO: Self = Self::Px(0.0);
    pub const DEFAULT: Self = Self::Auto;

    pub const fn from_px(value: &f32) -> Self {
        Self::Px(*value)
    }

    pub fn resolve_into_px(self, window_size: Size, element_size: f32) -> f32 {
        match self {
            Length::Auto => 0.0,
            Length::Px(val) => val,
            Length::Percent(val) => element_size * val / 100.0,
            Length::Vh(val) => window_size.height * val / 100.0,
            Length::Vw(val) => window_size.width * val / 100.0,
        }
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
            Length::Px(val) => taffy::LengthPercentageAuto::length(val),
            Length::Percent(val) => taffy::LengthPercentageAuto::percent(val / 100.0),
            Length::Vh(val) => {
                taffy::LengthPercentageAuto::length(window_size.height * val / 100.0)
            }
            Length::Vw(val) => taffy::LengthPercentageAuto::length(window_size.width * val / 100.0),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

pub struct Corners {
    pub top_left: Length,
    pub top_right: Length,
    pub bottom_right: Length,
    pub bottom_left: Length,
}

impl Corners {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Left,
    Top,
    Right,
    Bottom,
}

/// Type used to define padding, margin and border
#[derive(Debug, Copy, Clone)]
pub struct Edges {
    pub left: Length,
    pub right: Length,
    pub top: Length,
    pub bottom: Length,
}

impl Edges {
    pub const ZERO: Self = Self {
        left: Length::ZERO,
        right: Length::ZERO,
        top: Length::ZERO,
        bottom: Length::ZERO,
    };

    pub const DEFAULT: Self = Self {
        left: Length::ZERO,
        right: Length::ZERO,
        top: Length::ZERO,
        bottom: Length::ZERO,
    };

    pub const fn all(value: Length) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub const fn all_px(value: f32) -> Self {
        Self::all(Length::Px(value))
    }

    pub const fn all_percent(value: f32) -> Self {
        Self::all(Length::Percent(value))
    }

    pub const fn left(value: Length) -> Self {
        Self {
            left: value,
            ..Self::ZERO
        }
    }

    pub const fn left_px(value: f32) -> Self {
        Self::left(Length::Px(value))
    }

    pub const fn left_percent(value: f32) -> Self {
        Self::left(Length::Percent(value))
    }

    pub const fn right(value: Length) -> Self {
        Self {
            right: value,
            ..Self::ZERO
        }
    }

    pub const fn right_px(value: f32) -> Self {
        Self::right(Length::Px(value))
    }

    pub const fn right_percent(value: f32) -> Self {
        Self::right(Length::Percent(value))
    }

    pub const fn top(value: Length) -> Self {
        Self {
            top: value,
            ..Self::ZERO
        }
    }

    pub const fn top_px(value: f32) -> Self {
        Self::top(Length::Px(value))
    }

    pub const fn top_percent(value: f32) -> Self {
        Self::top(Length::Percent(value))
    }

    pub const fn bottom(value: Length) -> Self {
        Self {
            bottom: value,
            ..Self::ZERO
        }
    }

    pub const fn bottom_px(value: f32) -> Self {
        Self::bottom(Length::Px(value))
    }

    pub const fn bottom_percent(value: f32) -> Self {
        Self::bottom(Length::Percent(value))
    }
}

impl Default for Edges {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<T> ResolveInto<taffy::Rect<T>> for Edges
where
    Length: ResolveInto<T>,
{
    fn resolve_into(self, window_size: Size) -> taffy::Rect<T> {
        taffy::Rect {
            left: self.left.resolve_into(window_size),
            right: self.right.resolve_into(window_size),
            top: self.top.resolve_into(window_size),
            bottom: self.bottom.resolve_into(window_size),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativePoint {
    pub x: Length,
    pub y: Length,
}

impl RelativePoint {
    pub(crate) fn resolve_into_point(self, window_size: Size, bounds: Rect) -> Point {
        Point {
            x: self.x.resolve_into_px(window_size, bounds.width()) + bounds.left,
            y: self.y.resolve_into_px(window_size, bounds.height()) + bounds.top,
        }
    }
}
