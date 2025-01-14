use crate::{app::{BuildContext, Widget}, style::StyleBuilder};

use super::{Styled};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element;

	fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self> {
		let style_builder = f(StyleBuilder::default());
		Styled { view: self, style_builder }
	}

    fn as_any_view(self) -> AnyView
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