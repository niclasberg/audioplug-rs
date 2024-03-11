use std::ops::Range;
use crate::{core::{Color, Point, Rectangle, Shape, Size, Vector}, event::{KeyEvent, MouseButton}, keyboard::{Key, Modifiers}, text::TextLayout, Event, LayoutHint, MouseEvent, View, Widget};
use unicode_segmentation::{UnicodeSegmentation, GraphemeCursor};

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

impl View for TextBox {
    type Element = TextBoxWidget;

    fn build(self, _ctx: &mut crate::BuildContext) -> Self::Element {
        TextBoxWidget { 
            width: self.width,
            editor: Editor::new(""), 
            text_layout: TextLayout::new("", Color::BLACK, Size::ZERO), 
            cursor_on: true, 
            last_cursor_timestamp: 0.0
        }
    }
}

pub struct TextBoxWidget {
    width: f64,
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

const PADDING: f64 = 2.0;
const CURSOR_DELAY_SECONDS: f64 = 0.5;

impl Widget for TextBoxWidget {
    fn event(&mut self, event: crate::Event, ctx: &mut crate::EventContext<()>) {
        let rebuild_text_layout = |this: &mut Self, ctx: &mut crate::EventContext<()>| {
            this.text_layout = TextLayout::new(&this.editor.value, Color::BLACK, Size::INFINITY);
            ctx.request_render();
        };

        match event {
            Event::Keyboard(key_event) => {
                match key_event {
                    KeyEvent::KeyDown { key, modifiers, str } =>
                        match (key, str) {
                            (Key::BackSpace, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if self.editor.remove_word_left() {
                                    rebuild_text_layout(self, ctx);
                                }
                            }, 
                            (Key::BackSpace, _) => {
                                if self.editor.remove_left() {
                                    rebuild_text_layout(self, ctx);
                                }
                            },
                            (Key::Delete, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if self.editor.remove_word_right() {
                                    rebuild_text_layout(self, ctx);
                                }
                            },
                            (Key::Delete, _) => {
                                if self.editor.remove_right() {
                                    rebuild_text_layout(self, ctx);
                                }
                            },
                            (Key::Left, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if self.editor.move_word_left(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            }
                            (Key::Left, _) => {
                                if self.editor.move_left(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            },
                            (Key::Right, _) if modifiers.contains(Modifiers::CONTROL) => {
                                if self.editor.move_word_right(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            },
                            (Key::Right, _) => {
                                if self.editor.move_right(modifiers.contains(Modifiers::SHIFT)) {
                                    ctx.request_render();
                                }
                            },
                            (Key::C, _) if modifiers == Modifiers::CONTROL => {
                                if let Some(selected_text) = self.editor.selected_text() {
                                    ctx.set_clipboard(selected_text);
                                }
                            },
                            (Key::V, _) if modifiers == Modifiers::CONTROL => {
                                if let Some(text_to_insert) = ctx.get_clipboard() {
                                    self.editor.insert(text_to_insert.as_str());
                                    rebuild_text_layout(self, ctx);
                                }
                            },
                            (Key::X, _) if modifiers == Modifiers::CONTROL => {

                            },
                            (_, Some(str)) if !modifiers.contains(Modifiers::CONTROL) => {
                                self.editor.insert(str.as_str());
                                rebuild_text_layout(self, ctx);
                            }
                            _ => {}
                        },
                    _ => {}
                }
            },
            Event::Mouse(mouse) => match mouse {
                MouseEvent::Down { button, position } if button == MouseButton::LEFT => {
                    if let Some(new_cursor) = self.text_layout.text_index_at_point(position) {
                        if self.editor.set_cursor(new_cursor, false) {
                            ctx.request_render();
                        }
                    }
                },
                _ => {}
            },
            Event::AnimationFrame { timestamp } => {
                if timestamp - self.last_cursor_timestamp > CURSOR_DELAY_SECONDS {
                    self.cursor_on = !self.cursor_on;
                    ctx.request_render();
                    self.last_cursor_timestamp = timestamp;
                }
            }
            _ => {}
        };
    }

    fn layout(&mut self, constraint: crate::core::Constraint, ctx: &mut crate::LayoutContext) -> crate::core::Size {
        let size = self.text_layout.measure();
        let size = Size::new(self.width, size.height);
        let size = size + Size::new(PADDING*2.0, PADDING*2.0);

        constraint.clamp(size)
    }

    fn layout_hint(&self) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Fixed, LayoutHint::Flexible)
    }

    fn render(&mut self, ctx: &mut crate::RenderContext) {
        ctx.stroke(ctx.local_bounds(), Color::RED, 1.0);
        let bounds = ctx.local_bounds().shrink(PADDING);

        ctx.use_clip(bounds, |ctx| {
            if let Some(selection) = self.editor.selection() {
                let left = self.text_layout.point_at_text_index(selection.start);
                let right = self.text_layout.point_at_text_index(selection.end);
                let rect = Rectangle::from_points(bounds.top_left() + left, bounds.bottom_left() + right);
                ctx.fill(rect, Color::from_rgb8(68, 85, 90));
            }

            ctx.draw_text(&self.text_layout, bounds.position());
            
            if self.cursor_on {
                let cursor_point = self.text_layout.point_at_text_index(self.editor.position);
                let p0 = bounds.bottom_left() + cursor_point; 
                let p1 = bounds.top_left() + cursor_point;
                ctx.fill(Shape::line(p0, p1), Color::BLACK);
            }
        });
    }
}