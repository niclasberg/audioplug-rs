use crate::{app::{Accessor, BuildContext, Widget}, core::Color};

use super::{Background, Styled};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element;

    fn background(self, color: Color) -> Background<Self> {
        Background { view: self, color }
    }

	fn padding(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding(value)
	}

	fn padding_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_percent(value)
	}

	fn padding_left(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_left(value)
	}

	fn padding_left_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_left_percent(value)
	}

	fn padding_right(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_right(value)
	}

	fn padding_right_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_right_percent(value)
	}

	fn padding_top(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_top(value)
	}

	fn padding_top_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_top_percent(value)
	}

	fn padding_bottom(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_bottom(value)
	}

	fn padding_bottom_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).padding_bottom_percent(value)
	}

	fn width(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).width(value)
	}

	fn width_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).width_percent(value)
	}

	fn width_auto(self) -> Styled<Self> {
		Styled::new(self).width_auto()
	}

	fn min_width(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).min_width(value)
	}

	fn min_width_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).min_width_percent(value)
	}

	fn min_width_auto(self) -> Styled<Self> {
		Styled::new(self).min_width_auto()
	}

	fn max_width(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).max_width(value)
	}

	fn max_width_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).max_width_percent(value)
	}

	fn max_width_auto(self) -> Styled<Self> {
		Styled::new(self).max_width_auto()
	}

	fn height(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).height(value)
	}

	fn height_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).height_percent(value)
	}

	fn height_auto(self) -> Styled<Self> {
		Styled::new(self).height_auto()
	}

	fn min_height(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).min_height(value)
	}

	fn min_height_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).min_height_percent(value)
	}

	fn min_height_auto(self) -> Styled<Self> {
		Styled::new(self).min_height_auto()
	}

	fn max_height(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).max_height(value)
	}

	fn max_height_percent(self, value: impl Into<Accessor<f64>>) -> Styled<Self> {
		Styled::new(self).max_height_percent(value)
	}

	fn max_height_auto(self) -> Styled<Self> {
		Styled::new(self).max_height_auto()
	}

    fn as_any(self) -> AnyView
    where 
        Self: 'static 
    {
        Box::new(move |ctx| Box::new(ctx.build(self)))
    }
}


impl View for AnyView {
	type Element = Box<dyn Widget>;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		self(ctx)
	}
}