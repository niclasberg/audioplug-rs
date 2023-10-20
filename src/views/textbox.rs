use crate::{View, event::KeyEvent, text::TextLayout, core::{Size, Point, Color}, LayoutHint};

pub struct TextBox {

}

impl TextBox {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct TextBoxState {
    value: String,
    cursor: usize,
    text_layout: TextLayout
}

pub enum TextBoxMessage {

}

impl View for TextBox {
    type Message = TextBoxMessage;
    type State = TextBoxState;

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State {
        TextBoxState { value: "".to_owned(), cursor: 0, text_layout: TextLayout::new("", Size::ZERO) }
    }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {
        
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {
        match event {
            crate::Event::Keyboard(key_event) => {
                match key_event {
                    KeyEvent::Characters { str } => {
                        state.value.push_str(&str);
                        state.text_layout = TextLayout::new(&state.value, Size::ZERO);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> crate::core::Size {
        state.text_layout.set_max_size(constraint.max());
        let size = state.text_layout.measure();

        constraint.clamp(size)
    }

    fn layout_hint(&self, state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Flexible)
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.draw_text(&state.text_layout, Point::ZERO, Color::BLACK)
    }
}