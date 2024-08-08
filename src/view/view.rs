use crate::core::Color;

use super::{Background, BuildContext, Styled, Widget};

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

    /*fn as_any(self) -> Box<dyn AnyView> 
    where 
        Self: 'static 
    {
        Box::new(self)
    }*/
}

impl<W: Widget + 'static, F: FnOnce(&mut BuildContext) -> W> View for F {
    type Element = W;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        self(ctx)
    }
}

/*pub trait AnyView {
    fn dyn_build(self, ctx: &mut BuildContext) -> Box<dyn Widget>;
}

impl<V: View + 'static> AnyView for V {
    fn dyn_build(self, ctx: &mut BuildContext) -> Box<dyn Widget> {
        Box::new(self.build(ctx))
    }
}

impl View for Box<dyn AnyView> {
    type Element = Box<dyn Widget>;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        self.dyn_build(ctx)
    }
}*/