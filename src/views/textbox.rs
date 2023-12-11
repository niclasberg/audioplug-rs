use std::ops::Range;
use crate::{View, event::{KeyEvent, MouseButton}, text::TextLayout, core::{Size, Point, Shape, Color, Vector, Rectangle}, LayoutHint, keyboard::{Key, Modifiers}, Event, MouseEvent};
use unicode_segmentation::{UnicodeSegmentation, GraphemeIndices, GraphemeCursor};

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
    editor: Editor,
    text_layout: TextLayout,
    cursor_on: bool,
    last_cursor_timestamp: f64
}

struct Editor {
    value: String,
    position: usize,
    selection_start: Option<usize>,
}

impl Editor {
    fn new(string: &str) -> Self {
        Self {
            value: string.to_owned(),
            position: 0,
            selection_start: None
        }
    }

    fn selection(&self) -> Option<Range<usize>> {
        match self.selection_start {
            Some(start) if start < self.position => Some(start..self.position),
            Some(end) if end > self.position => Some(self.position..end),
            _ => None
        }
    }

    fn selected_text(&self) -> Option<&str> {
        self.selection().map(|range| {
            &self.value[range]
        })
    }

    fn set_cursor(&mut self, position: usize, select: bool) -> bool {
        let changed = position != self.position;
        self.position = position;
        changed
    }

    fn select_word_at(&mut self, index: usize) -> bool {

        true
    }

    fn prev(&self) -> Option<usize> {
        GraphemeCursor::new(self.position, self.value.len(), true)
            .prev_boundary(&self.value, 0).unwrap()
    }

    fn prev_word(&self) -> Option<usize> {
        self.value.unicode_word_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index < self.position)
            .last()
    }

    fn next(&self) -> Option<usize> {
        GraphemeCursor::new(self.position, self.value.len(), true)
            .next_boundary(&self.value, 0).unwrap()
    }

    fn next_word(&self) -> Option<usize> {
        self.value.unicode_word_indices()
            .map(|(index, _)| index)
            .skip_while(|index| *index <= self.position)
            .chain(std::iter::once(self.value.len()))
            .next()
    }

    fn move_left(&mut self, select: bool) -> bool {
        match (self.selection_start, select) {
            (Some(_), true) | (None, false) => {
                self.prev().is_some_and(|position| {
                    self.position = position;
                    true
                })
            },
            (Some(selection_start), false) => {
                self.position = self.position.min(selection_start);
                self.selection_start = None;
                true
            },
            (None, true) => {
                if let Some(position) = self.prev() {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                } else {
                    false
                }
            },
        }
    }

    fn move_word_left(&mut self, select: bool) -> bool {
        match (self.selection_start, select) {
            (Some(_), true) | (None, false) => {
                self.prev_word().is_some_and(|position| {
                    self.position = position;
                    true
                })
            },
            (Some(selection_start), false) => {
                self.position = self.position.min(selection_start);
                self.position = self.prev_word().unwrap_or(self.position);
                self.selection_start = None;
                true
            },
            (None, true) => {
                self.prev_word().is_some_and(|position| {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                })
            },
        }
    }

    fn move_right(&mut self, select: bool) -> bool {
        match (self.selection_start, select) {
            (Some(_), true) | (None, false) => {
                if let Some(position) = self.next() {
                    self.position = position;
                    true
                } else {
                    false
                }
            },
            (Some(selection_start), false) => {
                self.position = self.position.max(selection_start);
                self.selection_start = None;
                true
            },
            (None, true) => {
                if let Some(position) = self.next() {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                } else {
                    false
                }
            },
        }
    }

    fn move_word_right(&mut self, select: bool) -> bool {
        match (self.selection_start, select) {
            (Some(_), true) | (None, false) => {
                if let Some(position) = self.next_word() {
                    self.position = position;
                    true
                } else {
                    false
                }
            },
            (Some(selection_start), false) => {
                self.position = self.position.max(selection_start);
                self.position = self.next_word().unwrap_or(self.position);
                self.selection_start = None;
                true
            },
            (None, true) => {
                if let Some(position) = self.next_word() {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                } else {
                    false
                }
            },
        }
    }

    fn insert(&mut self, string: &str) -> bool {
        if let Some(selection) = self.selection() {
            self.value.replace_range(selection.clone(), string);
            self.selection_start = None;
            self.position = selection.start + string.len();
        } else {
            if self.position == self.value.len() {
                self.value.extend(string.chars());
                self.position += string.len();
            } else {
                self.value.insert_str(self.position, string);
                self.position += string.len();
            }
        }
        true
    }

    fn remove_selected(&mut self) -> bool {
        if let Some(selection) = self.selection() {
            self.position = selection.start;
            self.value.replace_range(selection, "");
            self.selection_start = None;
            true
        } else {
            false
        }
    }

    fn remove_left(&mut self) -> bool {
        self.remove_selected() || self.prev().is_some_and(|pos_left| {
            self.value.replace_range(pos_left..self.position, "");
            self.position = pos_left;
            true
        })
    }

    fn remove_right(&mut self) -> bool {
        self.remove_selected() || self.next().is_some_and(|pos_right| {
            self.value.replace_range(self.position..pos_right, "");
            true
        })
    }

    fn remove_word_left(&mut self) -> bool {
        self.remove_selected() || self.prev_word().is_some_and(|pos_left| {
            self.value.replace_range(pos_left..self.position, "");
            self.position = pos_left;
            true
        })
    }

    fn remove_word_right(&mut self) -> bool {
        self.remove_selected() || self.next_word().is_some_and(|pos_right| {
            self.value.replace_range(self.position..pos_right, "");
            true
        })
    }
}

pub enum TextBoxMessage {

}

const PADDING: f64 = 2.0;
const CURSOR_DELAY_SECONDS: f64 = 0.5;

impl View for TextBox {
    type Message = TextBoxMessage;
    type State = TextBoxState;

    fn build(&mut self, _ctx: &mut crate::BuildContext) -> Self::State {
        TextBoxState { 
            editor: Editor::new(""), 
            text_layout: TextLayout::new("", Color::BLACK, Size::ZERO), 
            cursor_on: true, 
            last_cursor_timestamp: 0.0
        }
    }

    fn rebuild(&mut self, _state: &mut Self::State, _ctx: &mut crate::BuildContext) {
        
    }

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<Self::Message>) {
        let rebuild_text_layout = |state: &mut Self::State, ctx: &mut crate::EventContext<Self::Message>| {
            state.text_layout = TextLayout::new(&state.editor.value, Color::BLACK, Size::INFINITY);
            ctx.request_render();
        };

        match event {
            Event::Keyboard(key_event) => {
                match key_event {
                    KeyEvent::KeyDown { key, modifiers, str } =>
                        match (key, str) {
                            (Key::BackSpace, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if state.editor.remove_word_left() {
                                    rebuild_text_layout(state, ctx);
                                }
                            }, 
                            (Key::BackSpace, _) => {
                                if state.editor.remove_left() {
                                    rebuild_text_layout(state, ctx);
                                }
                            },
                            (Key::Delete, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if state.editor.remove_word_right() {
                                    rebuild_text_layout(state, ctx);
                                }
                            },
                            (Key::Delete, _) => {
                                if state.editor.remove_right() {
                                    rebuild_text_layout(state, ctx);
                                }
                            },
                            (Key::Left, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if state.editor.move_word_left(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            }
                            (Key::Left, _) => {
                                if state.editor.move_left(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            },
                            (Key::Right, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if state.editor.move_word_right(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            },
                            (Key::Right, _) => {
                                if state.editor.move_right(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            },
                            (Key::C, _) if modifiers == Modifiers::CONTROL => {
                                if let Some(selected_text) = state.editor.selected_text() {
                                    ctx.set_clipboard(selected_text);
                                }
                            },
                            (Key::V, _) if modifiers == Modifiers::CONTROL => {
                                if let Some(text_to_insert) = ctx.get_clipboard() {
                                    state.editor.insert(text_to_insert.as_str());
                                    rebuild_text_layout(state, ctx);
                                }
                            },
                            (Key::X, _) if modifiers == Modifiers::CONTROL => {

                            },
                            (_, Some(str)) if !modifiers.contains(Modifiers::CONTROL) => {
                                state.editor.insert(str.as_str());
                                rebuild_text_layout(state, ctx);
                            }
                            _ => {}
                        },
                    _ => {}
                }
            },
            Event::Mouse(mouse) => match mouse {
                MouseEvent::Down { button, position } if button == MouseButton::LEFT => {
                    if let Some(new_cursor) = state.text_layout.text_index_at_point(position) {
                        if state.editor.set_cursor(new_cursor, false) {
                            ctx.request_render();
                        }
                    }
                },
                _ => {}
            },
            Event::AnimationFrame { timestamp } => {
                if timestamp - state.last_cursor_timestamp > CURSOR_DELAY_SECONDS {
                    state.cursor_on = !state.cursor_on;
                    ctx.request_render();
                    state.last_cursor_timestamp = timestamp;
                }
            }
            _ => {}
        };
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
        let bounds = ctx.local_bounds().shrink(PADDING);

        ctx.use_clip(bounds, |ctx| {
            if let Some(selection) = state.editor.selection() {
                let left = state.text_layout.point_at_text_index(selection.start);
                let right = state.text_layout.point_at_text_index(selection.end);
                let rect = Rectangle::from_points(bounds.top_left() + left, bounds.bottom_left() + right);
                ctx.fill(rect, Color::from_rgb8(68, 85, 90));
            }

            ctx.draw_text(&state.text_layout, bounds.position());
            
            if state.cursor_on {
                let cursor_point = state.text_layout.point_at_text_index(state.editor.position);
                let p0 = bounds.bottom_left() + cursor_point; 
                let p1 = bounds.top_left() + cursor_point;
                ctx.fill(Shape::line(p0, p1), Color::BLACK);
            }
        });
    }
}