use taffy::style_helpers::FromLength;

use crate::{Event, Id};
use crate::view::{ViewSequence, View, BuildContext, EventContext, LayoutContext};
use crate::core::{Alignment, Axis};

use super::{RenderContext, Widget, WidgetNode};

pub struct Column<VS: ViewSequence> {
    view_seq: VS,
    alignment: Alignment,
    spacing: f64,
}

impl<VS: ViewSequence> Column<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            alignment: Alignment::Center,
            spacing: 0.0
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

impl<VS: ViewSequence> View for Column<VS> {
    type Element = LinearLayoutWidget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        LinearLayoutWidget {
            widgets: self.view_seq.build(ctx),
            alignment: self.alignment,
            spacing: self.spacing,
            axis: taffy::FlexDirection::Column
        }
    }
}

pub struct Row<VS: ViewSequence> {
    view_seq: VS,
    alignment: Alignment,
    spacing: f64
}

impl<VS: ViewSequence> Row<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            alignment: Alignment::Center,
            spacing: 0.0
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

impl<VS: ViewSequence> View for Row<VS> {
    type Element = LinearLayoutWidget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        LinearLayoutWidget {
            widgets: self.view_seq.build(ctx),
            alignment: self.alignment,
            spacing: self.spacing,
            axis: taffy::FlexDirection::Row
        }
    }
}

pub struct LinearLayoutWidget {
    widgets: Vec<WidgetNode>,
    alignment: Alignment,
    spacing: f64,
    axis: taffy::FlexDirection,
}

impl Widget for LinearLayoutWidget {
    fn event(&mut self, event: Event, ctx: &mut EventContext) {
        for widget in self.widgets.iter_mut() {
            widget.event(event.clone(), ctx);
        }
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        ctx.compute_flexbox_layout(self, inputs)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        for widget in self.widgets.iter_mut() {
            widget.render(ctx);
        }
    }
    
    fn style(&self) -> taffy::Style {
        taffy::Style {
            flex_direction: self.axis,
            gap: taffy::Size::from_length(self.spacing as f32),
            ..Default::default()
        }
    }
    
    fn child_count(&self) -> usize { 
        self.widgets.len() 
    }

    fn get_child<'a>(&'a self, i: usize) -> &'a WidgetNode { 
        &self.widgets[i] 
    }
    
    fn get_child_mut<'a>(&'a mut self, i: usize) -> &'a mut WidgetNode { 
        &mut self.widgets[i] 
    }
}