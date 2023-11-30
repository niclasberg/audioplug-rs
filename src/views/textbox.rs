use crate::{View, event::{KeyEvent, MouseButton}, text::TextLayout, core::{Size, Point, Shape, Color}, LayoutHint, keyboard::{Key, Modifiers}, Event, MouseEvent};

pub struct TextBox {
    width: f64
}

impl TextBox {
    pub fn new() -> Self {
        Self {
            width: 100.0
        }
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
        TextBoxState { value: "".to_owned(), cursor: 0, text_layout: TextLayout::new("", Color::BLACK, Size::ZERO) }
    }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {
        
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {
        let modified = match event {
            Event::Keyboard(key_event) => {
                match key_event {
                    KeyEvent::KeyDown { key, modifiers } =>
                        match key {
                            Key::BackSpace if modifiers.contains(Modifiers::CONTROL) => {
                                if let Some(str) = state.value.as_str().get(state.cursor..) {
                                    state.value = str.to_owned();
                                    state.cursor = 0;
                                    true
                                } else {
                                    false
                                }
                            }, 
                            Key::BackSpace => {
                                if state.cursor > 0 {
                                    state.cursor -= 1;
                                    state.value.remove(state.cursor);
                                    true
                                } else {
                                    false
                                }
                            },
                            Key::Delete if modifiers.contains(Modifiers::CONTROL) => {
                                if let Some(str) = state.value.as_str().get(..state.cursor) {
                                    state.value = str.to_owned();
                                    state.cursor = state.value.len();
                                    true
                                } else {
                                    false
                                }
                            },
                            Key::Delete => {
                                if state.cursor < state.value.len() {
                                    state.value.remove(state.cursor);
                                    true
                                } else {
                                    false
                                }
                            },
                            Key::Left => {
                                state.cursor = state.cursor.saturating_sub(1);
                                true
                            },
                            Key::Right => {
                                state.cursor = (state.cursor + 1).min(state.value.len());
                                true
                            }
                            _ => false
                        },
                    KeyEvent::Characters { str } => {
                        state.value.insert_str(state.cursor, &str);
                        state.cursor += str.len();
                        true
                    },
                    _ => false
                }
            },
            Event::Mouse(mouse) => match mouse {
                MouseEvent::Down { button, position } if button == MouseButton::LEFT => {
                    if let Some(new_cursor) = state.text_layout.text_index_at_point(position) {
                        state.cursor = new_cursor;
                        true
                    } else {
                        false
                    }
                },
                _ => false
            },
            _ => false
        };

        if modified {
            state.text_layout = TextLayout::new(&state.value, Color::BLACK, Size::INFINITY);
            ctx.request_render();
        }
    }

    fn layout(&self, state: &mut Self::State, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> crate::core::Size {
        let size = state.text_layout.measure();
        let size = Size::new(self.width, size.height);
        let size = size + Size::new(PADDING*2.0, PADDING*2.0);

        constraint.clamp(size)
    }

    fn layout_hint(&self, _state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Flexible)
    }

    fn render(&self, state: &Self::State, ctx: &mut crate::RenderContext) {
        ctx.stroke(ctx.local_bounds(), Color::RED, 1.0);
        ctx.use_clip(ctx.local_bounds().shrink(PADDING), |ctx| {
            ctx.draw_text(&state.text_layout, Point::ZERO);
            if let Some(cursor_point) = state.text_layout.point_at_text_index(state.cursor) {
                let p0 = Point::new(cursor_point.x, ctx.local_bounds().bottom());
                let p1 = Point::new(cursor_point.x, ctx.local_bounds().top());
                println!("Cursor: {:?}->{:?}", p0, p1);
                ctx.fill(Shape::line(p0, p1), Color::BLACK);
            }
        });
    }
}