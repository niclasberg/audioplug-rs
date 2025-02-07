use taffy::{FlexDirection, FlexWrap};

use crate::{app::{Accessor, BuildContext, ViewSequence, Widget}, style::{DisplayStyle, FlexStyle, Length}};

use super::View;


pub struct Row<VS: ViewSequence> {
    view_seq: VS,
    spacing: Accessor<Length>,
    wrap: Accessor<FlexWrap>
}

impl<VS: ViewSequence> Row<VS> {
    pub fn new(view_seq: VS) -> Self {
        Self {
            view_seq,
            spacing: Accessor::Const(Length::ZERO),
            wrap: Accessor::Const(Default::default()),
        }
    }

    pub fn spacing(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.spacing = value.into();
        self
    }
}

impl<VS: ViewSequence> View for Row<VS> {
    type Element = ContainerWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        self.view_seq.build_seq(ctx);
        let flex_style = FlexStyle {
            gap: self.spacing.get_and_bind(ctx, |value, mut widget| {
                widget.flex_style.gap = value;
                widget.request_layout();
            }),
            wrap: self.wrap.get_and_bind(ctx, |value, mut widget| {
                widget.flex_style.wrap = value;
                widget.request_layout();
            }),
        };

        ContainerWidget {
            flex_style
        }
    }
}

pub struct Container<V> {
	view: V
}

impl<V> Container<V> {
	pub fn new(view: V) -> Self {
		Self {
			view
		}
	}
}

impl<V: View> View for Container<V> {
	type Element = ContainerWidget;

	fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
		ctx.add_child(self.view);
		ContainerWidget {

		}

	}
}

pub struct ContainerWidget {
	display_style: OwnedDisplayStyle
}

impl Widget for ContainerWidget {
	fn debug_label(&self) -> &'static str {
		"Container"
	}

	fn render(&mut self, ctx: &mut crate::app::RenderContext) {
		ctx.render_children()
	}

	fn display_style(&self) -> DisplayStyle {
		&self.display_style
	}
}