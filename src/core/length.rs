pub enum Length {
	Auto,
	Px(f64),
	Percent(f64)
}

impl Default for Length {
	fn default() -> Self {
		Self::Auto
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