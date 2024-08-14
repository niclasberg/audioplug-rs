use crate::{app::{BuildContext, Widget}, core::Color};

use super::{Background, Styled};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext) -> Box<dyn Widget>>;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext) -> Self::Element;

    fn background(self, color: Color) -> Background<Self> {
        Background { view: self, color }
    }

    fn with_style<F: Fn(&mut taffy::Style)>(self, f: F) -> Styled<Self, F> {
        Styled {
            view: self,
            style_function: f
        }
    }

    fn as_any(self) -> AnyView
    where 
        Self: 'static 
    {
        Box::new(move |ctx: &mut BuildContext| Box::new(self.build(ctx)))
    }
}


impl View for AnyView {
	type Element = Box<dyn Widget>;

	fn build(self, ctx: &mut BuildContext) -> Self::Element {
		self(ctx)
	}
}