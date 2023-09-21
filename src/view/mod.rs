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

pub trait View<Message>: Sized {
    type State;

    fn build(&self, view_id: &IdPath) -> Self::State;
    fn rebuild(&self, view_id: &IdPath, prev: &Self, state: &mut Self::State);
    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Message>);
    fn layout(&mut self, state: &Self::State, constraint: Constraint) -> Size;
    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut Renderer);
}

pub struct Map<Msg, V, F> {
    view: V,
    map: F,
    _phantom: PhantomData<Msg>
}

impl<Msg, V: View<Msg>, F, U> View<U> for Map<Msg, V, F>
where
    F: Fn(Msg) -> U
{
    type State = V::State;

    fn build(&self, view_id: &IdPath) -> Self::State {
        todo!()
    }

    fn rebuild(&self, view_id: &IdPath, prev: &Self, state: &mut Self::State) {
        todo!()
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Msg>) {
        todo!()
    }

    fn layout(&mut self, state: &Self::State, constraint: Constraint) -> Size {
        todo!()
    }

    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut Renderer) {
        todo!()
    }
}