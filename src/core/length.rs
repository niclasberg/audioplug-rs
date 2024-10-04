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