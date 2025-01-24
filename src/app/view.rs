use crate::{app::{BuildContext, Widget}, style::StyleBuilder};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element;

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