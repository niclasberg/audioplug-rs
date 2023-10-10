use crate::{event::Event, core::{Constraint, Size}, ViewMessage}; 

mod id;
mod any_view;
mod contexts;
mod view_node;
mod view_sequence;
mod shape;
pub use id::{Id, IdPath};
pub use any_view::*;
pub use contexts::{LayoutContext, EventContext, BuildContext, RenderContext};
pub use view_node::*;
pub use view_sequence::*;
pub use shape::*;


pub trait View: Sized {
	type Message: 'static;
    type State;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State;
    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext);
    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>);

    /// Layout the view and (possibly) its subviews
    /// The view is passed a constraint and returns the size it wants.
    /// If the view is supposed to fill as much of the available space as possible,
    /// the size can be INFINITY.
    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size;
    fn render(&self, state: &Self::State, ctx: &mut RenderContext);
    
    fn map<Msg, F>(self, f: F) -> Map<Self, F> 
    where 
        F: Fn(Self::Message) -> Msg 
    {
        Map { view: self, map: f }
    }
}

pub struct Map<V, F> {
    view: V,
    map: F
}

impl<V: View, F, Msg: 'static> View for Map<V, F>
where
    F: Fn(V::Message) -> Msg
{
	type Message = Msg;
    type State = V::State;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        self.view.build(ctx)
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        self.view.rebuild(state, ctx)
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>) {
        let mut messages: Vec<ViewMessage<V::Message>> = Vec::new();
        ctx.with_message_container(&mut messages, |ctx| {
            self.view.event(state, event, ctx);
        });

        ctx.messages.extend(messages.into_iter().map(|m| ViewMessage {
            view_id: m.view_id,
            message: (self.map)(m.message), 
        }));
    }

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        self.view.layout(state, constraint, ctx)
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        self.view.render(state, ctx)
    }
}