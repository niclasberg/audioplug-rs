use taffy::{FlexDirection, FlexWrap};

use crate::app::{Accessor, BuildContext, RenderContext, Widget, ViewSequence, View};
use crate::style::{DisplayStyle, FlexStyle, Length};
use crate::core::Alignment;

pub struct Flex<VS: ViewSequence> {
    view_seq: VS,
    direction: Accessor<FlexDirection>,
    alignment: Alignment,
    spacing: Accessor<Length>,
    wrap: Accessor<FlexWrap>
}

impl<VS: ViewSequence> Flex<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            alignment: Alignment::Center,
            spacing: Accessor::Const(Length::ZERO),
            wrap: Accessor::Const(Default::default()),
            direction: Accessor::Const(FlexDirection::default())
        }
    }

    pub fn row(view_seq: VS) -> Self {
        let mut this = Self::new(view_seq);
        this.direction = Accessor::Const(FlexDirection::Row);
        this
    }

    pub fn column(view_seq: VS) -> Self {
        let mut this = Self::new(view_seq);
        this.direction = Accessor::Const(FlexDirection::Column);
        this
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn direction(mut self, value: impl Into<Accessor<FlexDirection>>) -> Self {
        self.direction = value.into();
        self
    }

    pub fn spacing(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.spacing = value.into();
        self
    }
}

impl<VS: ViewSequence> View for Flex<VS> {
    type Element = FlexWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        self.view_seq.build_seq(ctx);
        let flex_style = FlexStyle {
            direction: ctx.get_and_track(self.direction, |value, mut widget| {
                widget.flex_style.direction = value;
                widget.request_layout();
            }),
            gap: ctx.get_and_track(self.spacing, |value, mut widget| {
                widget.flex_style.gap = value;
                widget.request_layout();
            }),
            wrap: ctx.get_and_track(self.wrap, |value, mut widget| {
                widget.flex_style.wrap = value;
                widget.request_layout();
            }),
        };

        FlexWidget {
            alignment: self.alignment,
            flex_style
        }
    }
}

pub struct FlexWidget {
    alignment: Alignment,
    flex_style: FlexStyle
}

impl Widget for FlexWidget {
	fn debug_label(&self) -> &'static str {
		"Flex"
	}

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.render_children();
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Flex(&self.flex_style)
    }
}