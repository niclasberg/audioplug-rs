use taffy::prelude::TaffyZero;

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
	Vw(f64)
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

impl From<taffy::LengthPercentageAuto> for Length {
	fn from(value: taffy::LengthPercentageAuto) -> Self {
		match value {
			taffy::LengthPercentageAuto::Length(val) => Length::Px(val as _),
			taffy::LengthPercentageAuto::Percent(val) => Length::Percent((val * 100.0) as _),
			taffy::LengthPercentageAuto::Auto => Length::Auto,
		}
	}
}

impl Into<taffy::LengthPercentageAuto> for Length {
	fn into(self) -> taffy::LengthPercentageAuto {
		match self {
			Length::Auto => taffy::LengthPercentageAuto::Auto,
			Length::Px(val) => taffy::LengthPercentageAuto::Length(val as _),
			Length::Percent(val) => taffy::LengthPercentageAuto::Percent((val / 100.0) as _),
			_ => todo!()
		}
	}
}

impl Into<taffy::LengthPercentage> for Length {
	fn into(self) -> taffy::LengthPercentage {
		match self {
			Length::Auto => taffy::LengthPercentage::ZERO,
			Length::Px(val) => taffy::LengthPercentage::Length(val as _),
			Length::Percent(val) => taffy::LengthPercentage::Percent((val / 100.0) as _),
			_ => todo!()
		}
	}
}

impl Into<taffy::Dimension> for Length {
	fn into(self) -> taffy::Dimension {
		match self {
			Length::Auto => taffy::Dimension::Auto,
			Length::Px(val) => taffy::Dimension::Length(val as _),
			Length::Percent(val) => taffy::Dimension::Percent((val / 100.0) as _),
			_ => todo!()
		}
	}
}