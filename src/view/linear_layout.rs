use taffy::style_helpers::FromLength;

use crate::app::{BuildContext, RenderContext, Widget};
use crate::view::{ViewSequence, View};
use crate::core::Alignment;

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

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        self.view_seq.build(ctx);
		ctx.set_style(taffy::Style {
            flex_direction: taffy::FlexDirection::Column,
            gap: taffy::Size::from_length(self.spacing as f32),
            display: taffy::Display::Flex,
            ..Default::default()
        });
		
        LinearLayoutWidget {
            alignment: self.alignment
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

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        self.view_seq.build(ctx);
		ctx.set_style(taffy::Style {
            flex_direction: taffy::FlexDirection::Row,
            gap: taffy::Size::from_length(self.spacing as f32),
            display: taffy::Display::Flex,
            ..Default::default()
        });
        LinearLayoutWidget {
            alignment: self.alignment,
        }
    }
}

pub struct LinearLayoutWidget {
    alignment: Alignment
}

impl Widget for LinearLayoutWidget {
	fn debug_label(&self) -> &'static str {
		"Flex"
	}

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.render_children();
    }
}