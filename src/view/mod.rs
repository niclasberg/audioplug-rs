use crate::{event::Event, core::{Constraint, Size, Color, Shape}, ViewMessage}; 

mod id;
mod any_view;
mod contexts;
mod view_node;
mod view_sequence;
mod component;
pub use id::{Id, IdPath};
pub use any_view::*;
pub use contexts::{LayoutContext, EventContext, BuildContext, RenderContext};
pub use view_node::*;
pub use view_sequence::*;

#[derive(Debug, PartialEq)]
pub enum LayoutHint {
    Fixed,
    Flexible
}

impl LayoutHint {
    pub fn combine(&self, other: &Self) -> Self {
        match (self, other) {
            (LayoutHint::Fixed, LayoutHint::Fixed) => LayoutHint::Fixed,
            _ => LayoutHint::Flexible,
        }
    }
}

pub trait View: Sized {
	type Message: 'static;
    type State;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State;
    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext);
    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>);

    /// Layout the view and (possibly) its subviews
    /// The view is passed a constraint and returns the size it wants.
    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size;

    /// Suggests how the size of the view is determined. 
    /// - A Fixed layout does not care about the suggested size passed into layout
    /// - The size of a Flexible layout depends on the size suggestion passed to layout
    fn layout_hint(&self, state: &Self::State) -> (LayoutHint, LayoutHint);
    fn render(&self, state: &Self::State, ctx: &mut RenderContext);
    
    fn map<Msg, F>(self, f: F) -> Map<Self, F> 
    where 
        F: Fn(Self::Message) -> Msg 
    {
        Map { view: self, map: f }
    }

    fn background(self, color: Color) -> Background<Self> {
        Background { view: self, color }
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

    fn layout_hint(&self, state: &Self::State) -> (LayoutHint, LayoutHint) {
        self.view.layout_hint(state)
    }
}

pub struct Background<V: View> {
    view: V,
    color: Color,
}

impl<V: View> View for Background<V> {
    type Message = V::Message;
    type State = V::State;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        self.view.build(ctx)
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        self.view.rebuild(state, ctx)
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Self::Message>) {
        self.view.event(state, event, ctx)
    }

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        self.view.layout(state, constraint, ctx)
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        ctx.fill(ctx.local_bounds(), self.color);
        self.view.render(state, ctx)
    }

    fn layout_hint(&self, state: &Self::State) -> (LayoutHint, LayoutHint) {
        self.view.layout_hint(state)
    }
}