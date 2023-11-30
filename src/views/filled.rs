use crate::{core::{Color, Shape, Point, Size, Rectangle}, View, LayoutHint};


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
    type Message =();
    type State = ();

    fn layout_hint(&self, _state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Fixed)
    }

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State { }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {}

    fn layout(&self, _state: &mut Self::State, constraint: crate::core::Constraint, _ctx: &mut crate::LayoutContext) -> Size {
        constraint.clamp(self.shape.bounds().size())
    }

    fn render(&self, _state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.fill(self.shape, self.color)
    }

    fn event(&mut self, _state: &mut Self::State, _event: crate::Event, _ctx: &mut crate::EventContext<Self::Message>) {}
}