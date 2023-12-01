use crate::{view::View, core::{Size, Color, Point, Shape}, text::TextLayout, LayoutHint};

pub struct Label {
    pub text: String,
	color: Color
}

pub struct LabelState {
    text_layout: TextLayout
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
    type Message = ();
    type State = LabelState;

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State {
        let text_layout = TextLayout::new(self.text.as_str(), self.color, Size::INFINITY);
        Self::State { text_layout }
    }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {

    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> Size {
        state.text_layout.set_max_size(constraint.max());
        let size = state.text_layout.measure();

        constraint.clamp(size)
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.draw_text(&state.text_layout, Point::ZERO)
    }

    fn event(&mut self, _state: &mut Self::State, _event: crate::Event, _ctx: &mut crate::EventContext<Self::Message>) {}

    fn layout_hint(&self, state: &Self::State) -> (LayoutHint, LayoutHint) {
        (LayoutHint::Flexible, LayoutHint::Flexible)
    }
}