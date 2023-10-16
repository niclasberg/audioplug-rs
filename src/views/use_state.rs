use std::marker::PhantomData;

use crate::{view::View, IdPath, BuildContext, EventContext, LayoutContext, ViewMessage, RenderContext};

pub struct UseState<V: View, State, FInit, FView, FUpdate> {
    f_init: FInit,
    f_view: FView,
    f_update: FUpdate,
    _phantom: PhantomData<fn(State) -> V>,
}   

pub struct UseStateState<T, V: View> {
    state: T,
    view: V,
    view_state: V::State,
}

impl<V: View, State, FInit, FView, FUpdate> View for UseState<V, State, FInit, FView, FUpdate> 
where
    FInit: Fn() -> State,
    FView: Fn(&State) -> V,
    FUpdate: Fn(V::Message, &mut State),
{
    type Message = ();
    type State = UseStateState<State, V>;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        let mut state = (self.f_init)();
        let mut view = (self.f_view)(&mut state);
        let view_state = view.build(ctx);
        UseStateState {
            state,
            view,
            view_state
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        let mut view = (self.f_view)(&mut state.state);
        view.rebuild(&mut state.view_state, ctx);
        state.view = view;
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut EventContext<Self::Message>) {
        let mut messages = Vec::<ViewMessage<V::Message>>::new();
        ctx.with_message_container(&mut messages, |ctx| {
            state.view.event(&mut state.view_state, event, ctx);
        });

        let mut updated = false;
        for message in messages {
            (self.f_update)(message.message, &mut state.state);
            updated = true;
        }

        if updated {
            ctx.request_rebuild();
        }
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut LayoutContext) -> crate::core::Size {
        state.view.layout(&mut state.view_state, constraint, ctx)
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        state.view.render(&state.view_state, ctx)
    }

    fn layout_hint(&self, state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        state.view.layout_hint(&state.view_state)
    }
}

pub fn use_state<V: View, State, FInit, FView, FUpdate>(init_state: FInit, view: FView, update: FUpdate) -> UseState<V, State, FInit, FView, FUpdate> 
where
    FInit: Fn() -> State,
    FView: Fn(&State) -> V,
    FUpdate: Fn(V::Message, &mut State)
{
    UseState {
        f_init: init_state,
        f_view: view,
        f_update: update,
        _phantom: PhantomData,
    }
}