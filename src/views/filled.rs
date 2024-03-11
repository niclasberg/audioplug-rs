use crate::{core::{Color, Point, Rectangle, Shape, Size}, LayoutHint, View, Widget};


pub trait Fill {
    fn fill(self, color: Color) -> Filled;
}

impl Fill for Shape {
    fn fill(self, color: Color) -> Filled {
        Filled { shape: self, color}
    }
}

impl Fill for Rectangle {
    fn fill(self, color: Color) -> Filled {
        Filled { shape: self.into(), color}
    }
}

pub struct Filled {
    shape: Shape,
    color: Color
}

impl View for Filled {
    type Element = Self;

    fn build(self, _ctx: &mut crate::BuildContext) -> Self { 
        self
    }
}

impl Widget for Filled {
    fn event(&mut self, event: crate::Event, ctx: &mut crate::EventContext<()>) {
        
    }

    fn layout(&mut self, constraint: crate::core::Constraint, _ctx: &mut crate::LayoutContext) -> Size {
        constraint.clamp(self.shape.bounds().size())
    }

    fn layout_hint(&self) -> (LayoutHint, LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Fixed)
    }

    fn render(&mut self, ctx: &mut crate::RenderContext) {
        ctx.fill(self.shape, self.color)
    }
}