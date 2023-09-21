use std::marker::PhantomData;

use crate::{view::View, IdPath};

pub struct UseState<V: View, State, FInit, F> {
    f_init: FInit,
    f: F,
    _phantom: PhantomData<fn(State) -> V>,
}   

pub struct UseStateState<T, V: View> {
    state: T,
    view: V,
    view_state: V::State,
}

impl<V: View, State, FInit, F> View for UseState<V, State, FInit, F> 
where
    State: PartialEq,
    FInit: Fn() -> State,
    F: Fn(&mut State) -> V
{
    type Message = V::Message;
    type State = UseStateState<State, V>;

    fn build(&self, id_path: &IdPath) -> Self::State {
        let mut state = (self.f_init)();
        let view = (self.f)(&mut state);
        let view_state = view.build(id_path);
        UseStateState {
            state,
            view,
            view_state
        }
    }

    fn rebuild(&self, id_path: &IdPath, prev: &Self, state: &mut Self::State) {
        let view = (self.f)(&mut state.state);
        view.rebuild(id_path, &state.view, &mut state.view_state);
    }

    fn event(&mut self, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {

    }

    fn layout(&mut self, constraint: crate::core::Constraint) -> crate::core::Size {
        todo!()
    }

    fn render(&self, bounds: crate::core::Rectangle, ctx: &mut crate::window::Renderer) {
        todo!()
    }
}