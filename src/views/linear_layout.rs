use taffy::{FlexDirection, FlexWrap};

use crate::app::{Accessor, BuildContext, RenderContext, Widget, ViewSequence, View};
use crate::style::{DisplayStyle, FlexStyle, Length};
use crate::core::Alignment;




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