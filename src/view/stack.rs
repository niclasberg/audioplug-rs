use crate::{core::{Alignment, Constraint, Size}, view::{EventContext, View}, Event, Id};

use super::{BuildContext, LayoutContext, LayoutHint, RenderContext, ViewSequence, Widget, WidgetNode};

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
            style: taffy::Style {
                ..Default::default()
            }
        }
    }
}

pub struct StackWidget {
    widgets: Vec<WidgetNode>,
    style: taffy::Style
}

impl Widget for StackWidget {
    fn event(&mut self, event: Event, ctx: &mut EventContext<()>) {
        for (i, node) in self.widgets.iter_mut().enumerate().rev() {
            ctx.forward_to_child(Id(i), event.clone(), |ctx, event| {
                node.widget.event(event, ctx);
            });
        }
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        taffy::compute_block_layout(ctx, 0, inputs)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        for (i, widget) in self.widgets.iter_mut().enumerate() {
            ctx.with_child(Id(i), |ctx| {
                widget.render(ctx);
            });
        }
    }
    
    fn style(&self) -> taffy::Style {
        self.style.clone()
    }

    fn child_count(&self) -> usize { self.widgets.len() }
    fn get_child<'a>(&'a self, i: usize) -> &'a WidgetNode { &self.widgets[i] }
    fn get_child_mut<'a>(&'a mut self, i: usize) -> &'a mut WidgetNode { &mut self.widgets[i] }
}