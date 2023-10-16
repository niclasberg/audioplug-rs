use crate::{core::{Size, Color, Point}, View, LayoutHint};

/// Represents a drawable shape
/// All shapes are centered at (0, 0)
pub enum Shape {
    Rect { size: Size },
    RoundedRect { size: Size, corner_radius: Size},
    Ellipse { radii: Size }
}

impl Shape {
    pub const fn rect(size: Size) -> Self {
        Shape::Rect { size }
    }

    pub const fn rounded_rect(size: Size, corner_radius: Size) -> Self {
        Self::RoundedRect { size, corner_radius }
    }

    pub const fn ellipse(radii: Size) -> Self {
        Shape::Ellipse { radii }
    }

    pub const fn circle(radius: f64) -> Self {
        Shape::Ellipse { radii: Size::new(radius, radius) }
    }

    pub fn size(&self) -> Size {
        match self {
            Shape::Rect { size } => *size,
            Shape::RoundedRect { size,.. } => *size,
            Shape::Ellipse { radii } => radii.scale(2.0),
        }
    }

    pub fn hit_test(&self, pos: Point) -> bool {
        match self {
            Shape::Rect { size } => rect_contains(*size, pos),
            Shape::RoundedRect { size, corner_radius } => todo!(),
            Shape::Ellipse { radii } => {
                if radii.width < f64::EPSILON || radii.height < f64::EPSILON {
                    false
                } else {
                    (pos.x / radii.width).powi(2) + (pos.y / radii.height).powi(2) <= 1.0
                }
            },
        }
    }

    pub fn fill(self, color: Color) -> FilledShape {
        FilledShape { shape: self, color }
    }
}

fn rect_contains(size: Size, pos: Point) -> bool {
    pos.x >= -size.width && pos.x <= size.width &&
    pos.y >= -size.height && pos.y <= size.height
}

pub struct FilledShape {
    shape: Shape,
    color: Color
}

impl View for FilledShape {
    type Message =();
    type State = ();

    fn layout_hint(&self, _state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Fixed)
    }

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State { }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {}

    fn layout(&self, _state: &mut Self::State, constraint: crate::core::Constraint, _ctx: &mut crate::LayoutContext) -> Size {
        constraint.clamp(self.shape.size())
    }

    fn render(&self, _state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.fill(&self.shape, Point::new(self.shape.size().width / 2.0, self.shape.size().height / 2.0), self.color)
    }

    fn event(&mut self, _state: &mut Self::State, _event: crate::Event, _ctx: &mut crate::EventContext<Self::Message>) {}
}