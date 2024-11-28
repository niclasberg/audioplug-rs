use super::Length;

/// Type used to define padding, margin and border
#[derive(Debug, Copy, Clone)]
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

impl Default for UiRect {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl Into<taffy::Rect<taffy::LengthPercentage>> for UiRect {
	fn into(self) -> taffy::Rect<taffy::LengthPercentage> {
		taffy::Rect {
			left: self.left.into(),
			right: self.right.into(),
			top: self.top.into(),
			bottom: self.bottom.into(),
		}
	}
}

impl Into<taffy::Rect<taffy::LengthPercentageAuto>> for UiRect {
	fn into(self) -> taffy::Rect<taffy::LengthPercentageAuto> {
		taffy::Rect {
			left: self.left.into(),
			right: self.right.into(),
			top: self.top.into(),
			bottom: self.bottom.into(),
		}
	}
}