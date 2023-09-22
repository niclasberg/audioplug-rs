use std::{any::Any, ops::{DerefMut, Deref}};
use crate::{Event, core::{Constraint, Size, Rectangle}, window::Renderer, LayoutContext};
use super::{IdPath, EventContext, View};

pub trait AnyView {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
    fn dyn_build(&mut self, id_path: &IdPath) -> Box<dyn Any>;
    fn dyn_rebuild(&mut self, id_path: &IdPath, prev: &Box<dyn AnyView>, state: &mut Box<dyn Any>);
    fn dyn_event(&mut self, state: &mut Box<dyn Any>, event: Event, ctx: &mut EventContext<Box<dyn Any>>);
    fn dyn_layout(&self, state: &Box<dyn Any>, constraint: Constraint, ctx: &mut LayoutContext) -> Size;
    fn dyn_render(&self, state: &Box<dyn Any>, bounds: Rectangle, ctx: &mut Renderer);
}

impl<V: View + 'static> AnyView for V {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&mut self, id_path: &IdPath) -> Box<dyn Any> {
        Box::new(self.build(id_path))
    }

    fn dyn_rebuild(&mut self, id_path: &IdPath, prev: &Box<dyn AnyView>, state: &mut Box<dyn Any>) {
        if let Some(prev) = prev.as_any().downcast_ref() {
            let state = state.downcast_mut().expect("Internal error, state has wrong type");
            self.rebuild(id_path, prev, state);
        } else {
            *state = Box::new(self.build(id_path));
            // Request layout / redraw
        }
    }

    fn dyn_event(&mut self, state: &mut Box<dyn Any>, event: Event, ctx: &mut EventContext<Box<dyn Any>>) {
        let state = state.downcast_mut().expect("Invalid state type");
        self.event(state, event, ctx.with_type_mut::<V::Message>());
    }

    fn dyn_layout(&self, state: &Box<dyn Any>, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        let state = state.downcast_ref().expect("Invalid state type");
        self.layout(state, constraint, ctx)
    }

    fn dyn_render(&self, state: &Box<dyn Any>, bounds: Rectangle, ctx: &mut Renderer) {
        let state = state.downcast_ref().expect("Invalid state type");
        self.render(state, bounds, ctx)
    }
}

impl View for Box<dyn AnyView> {
	type Message = Box<dyn Any>;
    type State = Box<dyn Any>;

    fn build(&mut self, id_path: &IdPath) -> Self::State {
        self.deref_mut().dyn_build(id_path)
    }

    fn rebuild(&mut self, view_id: &IdPath, prev: &Self, state: &mut Self::State) {
        self.deref_mut().dyn_rebuild(view_id, prev, state)
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Box<dyn Any>>) {
        self.deref_mut().dyn_event(state, event, ctx)
    }

    fn layout(&self, state: &Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        self.deref().dyn_layout(state, constraint, ctx)
    }

    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut Renderer) {
        self.deref().dyn_render(state, bounds, ctx)
    }
}