use std::marker::PhantomData;
use crate::{Event, RenderContext, LayoutHint};
use crate::view::{ViewSequence, View, BuildContext, EventContext, LayoutContext};
use crate::core::{Alignment, Constraint, Size, Vector};

pub struct Column<Msg: 'static, VS: ViewSequence<Msg>> {
    view_seq: VS,
    alignment: Alignment,
    spacing: f64,
    _phantom: PhantomData<Msg>
}

impl<Msg: 'static, VS: ViewSequence<Msg>> Column<Msg, VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            alignment: Alignment::Center,
            spacing: 0.0,
            _phantom: PhantomData
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<Msg: 'static, VS: ViewSequence<Msg>> View for Column<Msg, VS> {
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
        // layout all children given infinite height
        let sizes = {
            let child_constraint = Constraint { min_size: Size::ZERO, max_size: constraint.max().with_height(f64::INFINITY) };
            self.view_seq.layout(state, child_constraint, ctx)
        };

        let max_width: f64 = sizes.iter()
            .map(|size| size.width)
            .fold(0.0, |acc, width| acc.max(width));

        if constraint.max().width.is_finite() {
            let total_spacing = self.spacing * (self.view_seq.len().saturating_sub(1) as f64);
            let total_height: f64 = sizes.iter()
                .map(|size| size.height)
                .sum();

            let total_size = constraint.clamp(Size::new(max_width, total_spacing + total_height));
            let mut pos_y = self.alignment.compute_offset_y(total_size.height, total_height + total_spacing);
            
            for (node, size) in ctx.node.children.iter_mut().zip(sizes) {
                let pos_x = self.alignment.compute_offset_x(size.width, total_size.width);
                node.set_offset(Vector::new(pos_x, pos_y));
                node.set_size(size);

                pos_y += self.spacing + size.height;
            }

            total_size
        } else {
            let mut pos_y = 0.0;
            for (node, size) in ctx.node.children.iter_mut().zip(sizes) {
                let pos_x = self.alignment.compute_offset_x(size.width, max_width);
                node.set_offset(Vector::new(pos_x, pos_y));
                node.set_size(size);
                pos_y += self.spacing + size.height;
            }

            Size::new(max_width, pos_y)
        }
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        self.view_seq.render(state, ctx)
    }

    fn layout_hint(&self, state: &Self::State) -> (LayoutHint, LayoutHint) {
        self.view_seq.layout_hint(state)
    }
}