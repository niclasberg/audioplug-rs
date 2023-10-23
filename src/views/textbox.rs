use crate::{View, event::KeyEvent, text::TextLayout, core::{Size, Point, Color}, LayoutHint, Shape};

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

const PADDING: f64 = 2.0;

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
                        state.text_layout = TextLayout::new(&state.value, Size::INFINITY);
                        ctx.request_render();
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
        let size = size + Size::new(PADDING*2.0, PADDING*2.0);

        constraint.clamp(size)
    }

    fn layout_hint(&self, _state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Flexible)
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.stroke(&Shape::rect(ctx.local_bounds().size()), ctx.local_bounds().center(), Color::RED, 1.0);

        ctx.draw_text(&state.text_layout, Point::new(PADDING, PADDING), Color::BLACK)
    }
}