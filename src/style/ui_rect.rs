use super::Length;

/// Type used to define padding, margin and border
pub struct UiRect {
    pub left: Length,
    pub right: Length,
    pub top: Length,
    pub bottom: Length
}

impl UiRect {
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
}