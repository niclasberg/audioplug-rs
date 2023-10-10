use crate::{view::View, core::{Size, Color, Point}, text::TextLayout};

pub struct Label {
    pub text: String
}

pub struct LabelState {
    text_layout: TextLayout
}

impl Label {
    pub fn new(str: impl Into<String>) -> Self {
        Self { text: str.into() }
    }
}

impl View for Label {
    type Message = ();
    type State = LabelState;

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State {
        let text_layout = TextLayout::new(self.text.as_str(), Size::INFINITY);
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
        ctx.draw_text(&state.text_layout, Point::ZERO, Color::BLACK)
    }

    fn event(&mut self, _state: &mut Self::State, _event: crate::Event, _ctx: &mut crate::EventContext<Self::Message>) {}
}