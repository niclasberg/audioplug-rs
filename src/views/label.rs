use crate::{core::{Color, Point, Size}, text::TextLayout, view::View, LayoutHint, Widget};

pub struct Label {
    pub text: String,
	color: Color
}

impl Label {
    pub fn new(str: impl Into<String>) -> Self {
        Self { text: str.into(), color: Color::BLACK }
    }

	pub fn with_color(mut self, color: Color) -> Self {
		self.color = color;
		self
	}
}

impl View for Label {
    type Element = TextWidget;

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::Element {
        let text_layout = TextLayout::new(self.text.as_str(), self.color, Size::INFINITY);
        TextWidget { text_layout }
    }
}

pub struct TextWidget {
    text_layout: TextLayout
}

impl Widget for TextWidget {
    fn layout(&mut self, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> Size {
        self.text_layout.set_max_size(constraint.max());
        let size = self.text_layout.measure();

        constraint.clamp(size)
    }

    fn render(&mut self, ctx: &mut crate::RenderContext) {
        ctx.draw_text(&self.text_layout, Point::ZERO)
    }

    fn event(&mut self, _event: crate::Event, _ctx: &mut crate::EventContext<()>) {}

    fn layout_hint(&self) -> (LayoutHint, LayoutHint) {
        (LayoutHint::Flexible, LayoutHint::Flexible)
    }
}