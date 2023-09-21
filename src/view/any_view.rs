use std::{any::Any, ops::{DerefMut, Deref}};
use crate::{Event, core::{Constraint, Size, Rectangle}, window::Renderer};
use super::{IdPath, EventContext, View};

pub trait AnyView<Msg> {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
    fn dyn_build(&self, id_path: &IdPath) -> Box<dyn Any>;
    fn dyn_rebuild(&mut self, id_path: &IdPath, prev: &Box<dyn AnyView<Msg>>, state: &mut Box<dyn Any>);
    fn dyn_event(&mut self, state: &mut Box<dyn Any>, event: Event, ctx: &mut EventContext<Msg>);
    fn dyn_layout(&mut self, state: &Box<dyn Any>, constraint: Constraint) -> Size;
    fn dyn_render(&self, state: &Box<dyn Any>, bounds: Rectangle, ctx: &mut Renderer);
}

impl<Msg, V: View<Msg> + 'static> AnyView<Msg> for V {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&self, id_path: &IdPath) -> Box<dyn Any> {
        Box::new(self.build(id_path))
    }

    fn dyn_rebuild(&mut self, id_path: &IdPath, prev: &Box<dyn AnyView<Msg>>, state: &mut Box<dyn Any>) {
        if let Some(prev) = prev.as_any().downcast_ref() {
            let state = state.downcast_mut().expect("Internal error, state has wrong type");
            self.rebuild(id_path, prev, state);
        } else {
            *state = Box::new(self.build(id_path));
            // Request layout / redraw
        }
    }

    fn dyn_event(&mut self, state: &mut Box<dyn Any>, event: Event, ctx: &mut EventContext<Msg>) {
        let state = state.downcast_mut().expect("Invalid state type");
        self.event(state, event, ctx);
    }

    fn dyn_layout(&mut self, state: &Box<dyn Any>, constraint: Constraint) -> Size {
        let state = state.downcast_ref().expect("Invalid state type");
        self.layout(state, constraint)
    }

    fn dyn_render(&self, state: &Box<dyn Any>, bounds: Rectangle, ctx: &mut Renderer) {
        let state = state.downcast_ref().expect("Invalid state type");
        self.render(state, bounds, ctx)
    }
}

impl<Msg> View<Msg> for Box<dyn AnyView<Msg>> {
    type State = Box<dyn Any>;

    fn build(&self, id_path: &IdPath) -> Self::State {
        self.deref().dyn_build(id_path)
    }

    fn rebuild(&self, view_id: &IdPath, prev: &Self, state: &mut Self::State) {
        self.deref().dyn_rebuild(view_id, prev, state)
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Msg>) {
        self.deref_mut().dyn_event(state, event, ctx)
    }

    fn layout(&mut self, state: &Self::State, constraint: Constraint) -> Size {
        self.deref_mut().dyn_layout(state, constraint)
    }

    fn render(&self, state: &Self::State, bounds: Rectangle, ctx: &mut Renderer) {
        self.deref().dyn_render(state, bounds, ctx)
    }
}