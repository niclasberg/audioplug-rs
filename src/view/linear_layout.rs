use taffy::style_helpers::FromLength;

use crate::app::{BuildContext, EventContext, LayoutContext, RenderContext};
use crate::MouseEvent;
use crate::view::{ViewSequence, View};
use crate::core::Alignment;

use super::{EventStatus, Widget};

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
        self.view_seq.build(ctx);
        LinearLayoutWidget {
            //widgets: self.view_seq.build(ctx),
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
        self.view_seq.build(ctx);
        LinearLayoutWidget {
            //widgets: self.view_seq.build(ctx),
            alignment: self.alignment,
            spacing: self.spacing,
            axis: taffy::FlexDirection::Row
        }
    }
}

pub struct LinearLayoutWidget {
    alignment: Alignment,
    spacing: f64,
    axis: taffy::FlexDirection,
}

impl Widget for LinearLayoutWidget {
    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) -> EventStatus {
        let mut status = EventStatus::Ignored;
        /*for widget in self.widgets.iter_mut().rev() {
            status = widget.mouse_event(event.clone(), ctx);
            if status == EventStatus::Handled {
                break;
            }
        }*/
        status
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.render_children();
    }
    
    fn style(&self) -> taffy::Style {
        taffy::Style {
            flex_direction: self.axis,
            gap: taffy::Size::from_length(self.spacing as f32),
            display: taffy::Display::Flex,
            ..Default::default()
        }
    }
}