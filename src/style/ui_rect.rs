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

    pub const fn all_px(value: f64) -> Self {
        Self::all(Length::Px(value))
    }

    pub const fn all_percent(value: f64) -> Self {
        Self::all(Length::Percent(value))
    }

    pub const fn left(value: Length) -> Self {
        Self {
            left: value,
            ..Self::ZERO
        }
    }

    pub const fn left_px(value: f64) -> Self {
        Self::left(Length::Px(value))
    }

    pub const fn left_percent(value: f64) -> Self {
        Self::left(Length::Percent(value))
    }

    pub const fn right(value: Length) -> Self {
        Self {
            right: value,
            ..Self::ZERO
        }
    }

    pub const fn right_px(value: f64) -> Self {
        Self::right(Length::Px(value))
    }

    pub const fn right_percent(value: f64) -> Self {
        Self::right(Length::Percent(value))
    }

    pub const fn top(value: Length) -> Self {
        Self {
            top: value,
            ..Self::ZERO
        }
    }

    pub const fn top_px(value: f64) -> Self {
        Self::top(Length::Px(value))
    }

    pub const fn top_percent(value: f64) -> Self {
        Self::top(Length::Percent(value))
    }

    pub const fn bottom(value: Length) -> Self {
        Self {
            bottom: value,
            ..Self::ZERO
        }
    }

    pub const fn bottom_px(value: f64) -> Self {
        Self::bottom(Length::Px(value))
    }

    pub const fn bottom_percent(value: f64) -> Self {
        Self::bottom(Length::Percent(value))
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