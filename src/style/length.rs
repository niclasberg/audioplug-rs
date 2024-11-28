#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
	Auto,
	Px(f64),
	Percent(f64)
}

impl Length {
	pub const ZERO: Self = Self::Px(0.0);
	pub const DEFAULT: Self = Self::Auto;
}

impl Default for Length {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl From<taffy::LengthPercentageAuto> for Length {
	fn from(value: taffy::LengthPercentageAuto) -> Self {
		match value {
			taffy::LengthPercentageAuto::Length(val) => Length::Px(val as _),
			taffy::LengthPercentageAuto::Percent(val) => Length::Percent(val as _),
			taffy::LengthPercentageAuto::Auto => Length::Auto,
		}
	}
}

impl Into<taffy::LengthPercentageAuto> for Length {
	fn into(self) -> taffy::LengthPercentageAuto {
		match self {
			Length::Auto => taffy::LengthPercentageAuto::Auto,
			Length::Px(val) => taffy::LengthPercentageAuto::Length(val as _),
			Length::Percent(val) => taffy::LengthPercentageAuto::Percent(val as _),
		}
	}
}