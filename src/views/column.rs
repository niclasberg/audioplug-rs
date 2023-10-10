use std::marker::PhantomData;
use crate::{Event, RenderContext};
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

    fn layout(&self, state: &Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        // layout all children given infinite width
        let sizes = {
            let child_constraint = Constraint { min_size: Size::ZERO, max_size: constraint.max().with_width(f64::INFINITY) };
            self.view_seq.layout(state, child_constraint, ctx)
        };

        let max_height: f64 = sizes.iter()
            .map(|size| size.height)
            .fold(0.0, |acc, height| acc.max(height));

        if constraint.max().width.is_finite() {
            let total_spacing = self.spacing * (self.view_seq.len().saturating_sub(1) as f64);
            let total_width: f64 = sizes.iter()
                .map(|size| size.width)
                .sum();

            let total_size = constraint.clamp(Size::new(total_spacing + total_width, max_height));
            let mut pos_x = self.alignment.compute_offset_x(total_width + total_spacing, total_size.width);
            
            for (node, size) in ctx.node.children.iter_mut().zip(sizes) {
                let pos_y = self.alignment.compute_offset_y(size.height, total_size.height);
                node.set_offset(Vector::new(pos_x, pos_y));
                node.set_size(size);

                pos_x += self.spacing + size.width;
            }

            total_size
        } else {
            let mut pos_x = 0.0;
            for (node, size) in ctx.node.children.iter_mut().zip(sizes) {
                let pos_y = self.alignment.compute_offset_y(size.height, max_height);
                node.set_offset(Vector::new(pos_x, pos_y));
                node.set_size(size);
                pos_x += self.spacing + size.width;
            }

            Size::new(pos_x, max_height)
        }
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        self.view_seq.render(state, ctx)
    }
}