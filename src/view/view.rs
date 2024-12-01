use crate::{app::{Accessor, BuildContext, Widget}, core::{Border, Color}, style::StyleBuilder};

use super::{Background, Styled};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element;

    fn background(self, color: impl Into<Accessor<Color>>) -> Background<Self> {
        Background { view: self, fill: Some(color.into()), border: None }
    }

	fn border(self, border: impl Into<Accessor<Border>>) -> Background<Self> {
        Background { view: self, fill: None, border: Some(border.into()) }
    }

	fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self> {
		let style_builder = f(StyleBuilder::default());
		Styled { view: self, style_builder }
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