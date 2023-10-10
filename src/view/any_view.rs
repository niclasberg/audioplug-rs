use std::{any::Any, ops::{DerefMut, Deref}};
use crate::{Event, core::{Constraint, Size}, RenderContext, LayoutContext, BuildContext, ViewMessage};
use super::{EventContext, View};

pub trait AnyView {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
    fn dyn_build(&mut self, ctx: &mut BuildContext) -> Box<dyn Any>;
    fn dyn_rebuild(&mut self, state: &mut Box<dyn Any>, ctx: &mut BuildContext);
    fn dyn_event(&mut self, state: &mut Box<dyn Any>, event: Event, ctx: &mut EventContext<Box<dyn Any>>);
    fn dyn_layout(&self, state: &mut Box<dyn Any>, constraint: Constraint, ctx: &mut LayoutContext) -> Size;
    fn dyn_render(&self, state: &Box<dyn Any>, ctx: &mut RenderContext);
}

impl<V: View + 'static> AnyView for V {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&mut self, ctx: &mut BuildContext) -> Box<dyn Any> {
        Box::new(self.build(ctx))
    }

    fn dyn_rebuild(&mut self, state: &mut Box<dyn Any>, ctx: &mut BuildContext) {
        if let Some(state) = state.downcast_mut() {
            self.rebuild(state, ctx);
        } else {
            *state = Box::new(self.build(ctx));
            // Request layout / redraw
        }
    }

    fn dyn_event(&mut self, state: &mut Box<dyn Any>, event: Event, ctx: &mut EventContext<Box<dyn Any>>) {
        let state = state.downcast_mut().expect("Invalid state type");
        let mut messages: Vec<ViewMessage<V::Message>> = Vec::new();
        ctx.with_message_container(&mut messages, |ctx| {
            self.event(state, event, ctx);
        });

        for message in messages {
            ctx.publish_message(Box::new(message));
        }
    }

    fn dyn_layout(&self, state: &mut Box<dyn Any>, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        let state = state.downcast_mut().expect("Invalid state type");
        self.layout(state, constraint, ctx)
    }

    fn dyn_render(&self, state: &Box<dyn Any>, ctx: &mut RenderContext) {
        let state = state.downcast_ref().expect("Invalid state type");
        self.render(state, ctx)
    }
}

impl View for Box<dyn AnyView> {
	type Message = Box<dyn Any>;
    type State = Box<dyn Any>;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        self.deref_mut().dyn_build(ctx)
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        self.deref_mut().dyn_rebuild(state, ctx)
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Box<dyn Any>>) {
        self.deref_mut().dyn_event(state, event, ctx)
    }

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        self.deref().dyn_layout(state, constraint, ctx)
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        self.deref().dyn_render(state, ctx)
    }
}