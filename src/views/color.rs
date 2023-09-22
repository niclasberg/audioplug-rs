use crate::{core::{Color, Constraint, Size, Rectangle}, View, window::Renderer, LayoutContext};

impl View for Color {
	type Message = ();
    type State = ();
    
    fn build(&mut self, view_id: &crate::IdPath) -> Self::State { () }
    fn rebuild(&mut self, view_id: &crate::IdPath, prev: &Self, state: &mut Self::State) { }

    fn layout(&self, _state: &Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        constraint.max()
    }

    fn render(&self, _state: &Self::State, bounds: Rectangle, ctx: &mut Renderer) {
        ctx.fill_rectangle(bounds, *self)
    }
}
