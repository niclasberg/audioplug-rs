use crate::{core::{Color, Constraint, Size, Rectangle}, View, Event, EventContext, window::Renderer};

impl View<()> for Color {
    type State = ();
    
    fn build(&self, view_id: &crate::IdPath) -> Self::State { () }
    fn rebuild(&self, view_id: &crate::IdPath, prev: &Self, state: &mut Self::State) { }
    fn event(&mut self, _state: &mut Self::State, _event: Event, _ctx: &mut EventContext<()>) {}

    fn layout(&mut self, _state: &Self::State, constraint: Constraint) -> Size {
        constraint.max()
    }

    fn render(&self, _state: &Self::State, bounds: Rectangle, ctx: &mut Renderer) {
        ctx.fill_rectangle(bounds, *self)
    }
}
