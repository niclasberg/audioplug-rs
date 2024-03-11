use crate::{core::{Alignment, Constraint, Size}, view::{EventContext, View}, BuildContext, Event, Id, LayoutContext, LayoutHint, RenderContext, ViewSequence, Widget};

pub struct Stack<VS> {
    view_seq: VS,
    alignment: Alignment
}

impl<VS: ViewSequence> Stack<VS> {
    pub fn new(views: VS) -> Self {
        Self {
            view_seq: views,
            alignment: Alignment::Center
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<VS: ViewSequence> View for Stack<VS> {
	type Element = StackWidget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        let widgets = self.view_seq.build(ctx);
        StackWidget {
            widgets,
            alignment: self.alignment
        }
    }
}

struct StackWidget {
    widgets: Vec<Box<dyn Widget>>,
    alignment: Alignment
}

impl Widget for StackWidget {
    fn event(&mut self, event: Event, ctx: &mut EventContext<()>) {
        for (i, widget) in self.widgets.iter_mut().enumerate().rev() {
            ctx.forward_to_child(Id(i), event, |ctx| {
                widget.event(event, ctx);
            });
        }
    }

    fn layout(&mut self, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
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