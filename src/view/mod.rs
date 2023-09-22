use std::marker::PhantomData;

use crate::{event::Event, window::Renderer, core::{Constraint, Size, Rectangle}}; 

mod id;
mod any_view;
mod contexts;
mod view_node;
mod view_sequence;
pub use id::{Id, IdPath};
pub use any_view::*;
pub use contexts::{LayoutContext, EventContext};
pub use view_node::*;
pub use view_sequence::*;

pub trait View: Sized {
	type Message;
    type State;

    fn build(&mut self, view_id: &IdPath) -> Self::State;
    fn rebuild(&mut self, view_id: &IdPath, prev: &Self, state: &mut Self::State);
    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>) {}
    fn layout(&self, state: &Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size;
    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut Renderer);
}

pub struct Map<V, F> {
    view: V,
    map: F
}

impl<V: View, F, U> View for Map<V, F>
where
    F: Fn(V::Message) -> U
{
	type Message = U;
    type State = V::State;

    fn build(&mut self, view_id: &IdPath) -> Self::State {
        todo!()
    }

    fn rebuild(&mut self, view_id: &IdPath, prev: &Self, state: &mut Self::State) {
        todo!()
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>) {
        todo!()
    }

    fn layout(&self, state: &Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        todo!()
    }

    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut Renderer) {
        todo!()
    }
}