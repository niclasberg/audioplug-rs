use crate::{core::{Size, Color, Point}, View};

pub enum Shape {
    Rect { size: Size },
    RoundedRect { size: Size, corner_radius: Size},
    Ellipse {}
}

impl Shape {
    pub fn rect(size: Size) -> Self {
        Shape::Rect { size }
    }

    pub fn rounded_rect(size: Size, corner_radius: Size) -> Self {
        Self::RoundedRect { size, corner_radius }
    }

    pub fn size(&self) -> Size {
        match self {
            Shape::Rect { size } => *size,
            Shape::RoundedRect { size,.. } => *size,
            Shape::Ellipse {  } => todo!(),
        }
    }

    pub fn fill(self, color: Color) -> FilledShape {
        FilledShape { shape: self, color }
    }
}

pub struct FilledShape {
    shape: Shape,
    color: Color
}

impl View for FilledShape {
    type Message =();
    type State = ();

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State { }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {}

    fn layout(&self, _state: &mut Self::State, constraint: crate::core::Constraint, _ctx: &mut crate::LayoutContext) -> Size {
        constraint.clamp(self.shape.size())
    }

    fn render(&self, _state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.fill(&self.shape, Point::ZERO, self.color)
    }

    fn event(&mut self, _state: &mut Self::State, _event: crate::Event, _ctx: &mut crate::EventContext<Self::Message>) {}
}