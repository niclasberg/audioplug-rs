use std::marker::PhantomData;
use crate::{view::{View, EventContext}, core::{Size, Rectangle, Constraint, Alignment}, ViewSequence, Event, LayoutContext, BuildContext, RenderContext, LayoutHint};

pub struct Stack<Msg: 'static, VS: ViewSequence<Msg>> {
    view_seq: VS,
    alignment: Alignment,
    _phantom: PhantomData<Msg>
}

impl<Msg: 'static, VS: ViewSequence<Msg>> Stack<Msg, VS> {
    pub fn new(views: VS) -> Self {
        Self {
            view_seq: views,
            alignment: Alignment::Center,
            _phantom: PhantomData,
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<Msg: 'static, VS: ViewSequence<Msg>> View for Stack<Msg, VS> {
    type Message = Msg;
	type State = VS::State;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        self.view_seq.build(ctx)
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        self.view_seq.rebuild(state, ctx);
    }

    fn event(&mut self, state: &mut Self::State, event: Event, ctx: &mut EventContext<Msg>) {
        self.view_seq.event(state, event, ctx);
    }

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        let sizes = self.view_seq.layout(state, constraint.with_min(Size::ZERO), ctx);
        let max_size = sizes.iter().fold(Size::ZERO, |acc, x| acc.max(x));
        let max_size = constraint.clamp(max_size);

        for (node, size) in ctx.node.children.iter_mut().zip(sizes) {
            node.set_size(size);
            node.set_offset(self.alignment.compute_offset(size, max_size));
        }

        max_size
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        self.view_seq.render(state, ctx)
    }

    fn layout_hint(&self, state: &Self::State) -> (LayoutHint, LayoutHint) {
        self.view_seq.layout_hint(state)
    }
}