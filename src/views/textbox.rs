use crate::app::{
    Accessor, AnimationContext, AppState, BuildContext, EventContext, EventStatus,
    MouseEventContext, RenderContext, Shape, StatusChange, TextLayout, View, Widget,
};
use crate::core::{Color, Key, Modifiers, Rectangle, Size};
use crate::event::{KeyEvent, MouseButton};
use crate::style::{AvailableSpace, DisplayStyle, Length, Measure, Style, UiRect};
use crate::MouseEvent;
use std::ops::Range;
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

pub struct TextBox {
    width: f64,
    input_changed_fn: Option<Box<dyn Fn(&mut AppState, &str)>>,
    value: Option<Accessor<String>>,
}

impl TextBox {
    pub fn new() -> Self {
        Self {
            width: 100.0,
            input_changed_fn: None,
            value: None,
        }
    }

    pub fn on_input(mut self, f: impl Fn(&mut AppState, &str) + 'static) -> Self {
        self.input_changed_fn = Some(Box::new(f));
        self
    }

    pub fn value(mut self, value: impl Into<Accessor<String>>) -> Self {
        self.value = Some(value.into());
        self
    }
}

impl View for TextBox {
    type Element = TextBoxWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        cx.set_focusable(true);

        {
            cx.set_style(Style {
                padding: UiRect::all(Length::Px(2.0)),
                border: Length::Px(1.0),
                ..Default::default()
            });
        }

        let text_layout = if let Some(value) = self.value {
            let text = value.get_and_bind(cx, |value, mut widget| {
                if value != widget.value {
                    widget.value = value;
                    widget.rebuild_text_layout();
                    widget.request_render();
                }
            });
            TextLayout::new(&text, Color::BLACK, Size::INFINITY)
        } else {
            TextLayout::new("", Color::BLACK, Size::INFINITY)
        };

        TextBoxWidget {
            width: self.width,
            value: "".to_owned(),
            text_layout,
            cursor_on: true,
            position: 0,
            selection_start: None,
            last_cursor_timestamp: 0.0,
            is_mouse_selecting: false,
            input_changed_fn: self.input_changed_fn,
        }
    }
}

pub struct TextBoxWidget {
    width: f64,
    value: String,
    text_layout: TextLayout,
    cursor_on: bool,
    position: usize,
    selection_start: Option<usize>,
    last_cursor_timestamp: f64,
    is_mouse_selecting: bool,
    input_changed_fn: Option<Box<dyn Fn(&mut AppState, &str)>>,
}

impl TextBoxWidget {
    fn selection(&self) -> Option<Range<usize>> {
        match self.selection_start {
            Some(start) if start < self.position => Some(start..self.position),
            Some(end) if end > self.position => Some(self.position..end),
            _ => None,
        }
    }

    fn selected_text(&self) -> Option<&str> {
        self.selection().map(|range| &self.value[range])
    }

    fn set_caret_position(&mut self, position: usize, select: bool) -> bool {
        let changed = position != self.position;
        if select {
            self.selection_start = self.selection_start.or(Some(self.position));
        } else {
            self.selection_start = None;
        }
        self.position = position;
        changed
    }

    fn select_word_at(&mut self, index: usize) -> bool {
        true
    }

    fn prev(&self) -> Option<usize> {
        GraphemeCursor::new(self.position, self.value.len(), true)
            .prev_boundary(&self.value, 0)
            .unwrap()
    }

    fn prev_word(&self) -> Option<usize> {
        self.value
            .unicode_word_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index < self.position)
            .last()
    }

    fn next(&self) -> Option<usize> {
        GraphemeCursor::new(self.position, self.value.len(), true)
            .next_boundary(&self.value, 0)
            .unwrap()
    }

    fn next_word(&self) -> Option<usize> {
        self.value
            .unicode_word_indices()
            .map(|(index, _)| index)
            .skip_while(|index| *index <= self.position)
            .chain(std::iter::once(self.value.len()))
            .next()
    }

    fn move_left(&mut self, select: bool) -> bool {
        match (self.selection_start, select) {
            (Some(_), true) | (None, false) => self.prev().is_some_and(|position| {
                self.position = position;
                true
            }),
            (Some(selection_start), false) => {
                self.position = self.position.min(selection_start);
                self.selection_start = None;
                true
            }
            (None, true) => {
                if let Some(position) = self.prev() {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn move_word_left(&mut self, select: bool) -> bool {
        match (self.selection_start, select) {
            (Some(_), true) | (None, false) => self.prev_word().is_some_and(|position| {
                self.position = position;
                true
            }),
            (Some(selection_start), false) => {
                self.position = self.position.min(selection_start);
                self.position = self.prev_word().unwrap_or(self.position);
                self.selection_start = None;
                true
            }
            (None, true) => self.prev_word().is_some_and(|position| {
                self.selection_start = Some(self.position);
                self.position = position;
                true
            }),
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
            }
            (Some(selection_start), false) => {
                self.position = self.position.max(selection_start);
                self.selection_start = None;
                true
            }
            (None, true) => {
                if let Some(position) = self.next() {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                } else {
                    false
                }
            }
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
            }
            (Some(selection_start), false) => {
                self.position = self.position.max(selection_start);
                self.position = self.next_word().unwrap_or(self.position);
                self.selection_start = None;
                true
            }
            (None, true) => {
                if let Some(position) = self.next_word() {
                    self.selection_start = Some(self.position);
                    self.position = position;
                    true
                } else {
                    false
                }
            }
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
        self.remove_selected()
            || self.prev().is_some_and(|pos_left| {
                self.value.replace_range(pos_left..self.position, "");
                self.position = pos_left;
                true
            })
    }

    fn remove_right(&mut self) -> bool {
        self.remove_selected()
            || self.next().is_some_and(|pos_right| {
                self.value.replace_range(self.position..pos_right, "");
                true
            })
    }

    fn remove_word_left(&mut self) -> bool {
        self.remove_selected()
            || self.prev_word().is_some_and(|pos_left| {
                self.value.replace_range(pos_left..self.position, "");
                self.position = pos_left;
                true
            })
    }

    fn remove_word_right(&mut self) -> bool {
        self.remove_selected()
            || self.next_word().is_some_and(|pos_right| {
                self.value.replace_range(self.position..pos_right, "");
                true
            })
    }

    fn rebuild_text_layout(&mut self) {
        self.text_layout = TextLayout::new(self.value.as_str(), Color::BLACK, Size::INFINITY);
    }
}

const CURSOR_DELAY_SECONDS: f64 = 0.5;

impl Measure for TextBoxWidget {
    fn measure(&self, _style: &Style, _width: AvailableSpace, _height: AvailableSpace) -> Size {
        let size = self.text_layout.measure();
        Size::new(self.width, size.height)
    }
}

impl Widget for TextBoxWidget {
    fn debug_label(&self) -> &'static str {
        "TextBox"
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        let rebuild_text_layout = |this: &mut Self, ctx: &mut EventContext| {
            this.rebuild_text_layout();
            ctx.request_render();
            if let Some(f) = &this.input_changed_fn {
                f(ctx.app_state_mut(), &this.value);
            }
        };

        match event {
            KeyEvent::KeyDown {
                key,
                modifiers,
                str,
            } => match (key, str) {
                (Key::BackSpace, _) if modifiers.contains(Modifiers::CONTROL) => {
                    if self.remove_word_left() {
                        rebuild_text_layout(self, ctx);
                    }
                    EventStatus::Handled
                }
                (Key::BackSpace, _) => {
                    if self.remove_left() {
                        rebuild_text_layout(self, ctx);
                    }
                    EventStatus::Handled
                }
                (Key::Delete, _) if modifiers.contains(Modifiers::CONTROL) => {
                    if self.remove_word_right() {
                        rebuild_text_layout(self, ctx);
                    }
                    EventStatus::Handled
                }
                (Key::Delete, _) => {
                    if self.remove_right() {
                        rebuild_text_layout(self, ctx);
                    }
                    EventStatus::Handled
                }
                (Key::Left, _) if modifiers.contains(Modifiers::CONTROL) => {
                    if self.move_word_left(modifiers.contains(Modifiers::SHIFT)) {
                        ctx.request_render();
                    }
                    EventStatus::Handled
                }
                (Key::Left, _) => {
                    if self.move_left(modifiers.contains(Modifiers::SHIFT)) {
                        ctx.request_render();
                    }
                    EventStatus::Handled
                }
                (Key::Right, _) if modifiers.contains(Modifiers::CONTROL) => {
                    if self.move_word_right(modifiers.contains(Modifiers::SHIFT)) {
                        ctx.request_render();
                    }
                    EventStatus::Handled
                }
                (Key::Right, _) => {
                    if self.move_right(modifiers.contains(Modifiers::SHIFT)) {
                        ctx.request_render();
                    }
                    EventStatus::Handled
                }
                (Key::C, _) if modifiers == Modifiers::CONTROL => {
                    if let Some(selected_text) = self.selected_text() {
                        ctx.clipboard().set_text(selected_text);
                    }
                    EventStatus::Handled
                }
                (Key::V, _) if modifiers == Modifiers::CONTROL => {
                    if let Some(text_to_insert) = ctx.clipboard().get_text() {
                        self.insert(text_to_insert.as_str());
                        rebuild_text_layout(self, ctx);
                    }
                    EventStatus::Handled
                }
                (Key::X, _) if modifiers == Modifiers::CONTROL => EventStatus::Handled,
                (Key::Tab, _) | (Key::Escape, _) | (Key::Enter, _) => EventStatus::Ignored,
                (_, Some(str)) if !modifiers.contains(Modifiers::CONTROL) => {
                    self.insert(str.as_str());
                    rebuild_text_layout(self, ctx);
                    EventStatus::Handled
                }
                _ => EventStatus::Ignored,
            },
            _ => EventStatus::Ignored,
        }
    }

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down {
                button, position, ..
            } if button == MouseButton::LEFT => {
                ctx.capture_mouse();
                if let Some(new_cursor) = self
                    .text_layout
                    .text_index_at_point(position - ctx.bounds().top_left())
                {
                    self.is_mouse_selecting = true;
                    if self.set_caret_position(new_cursor, false) {
                        ctx.request_render();
                    }
                }
                EventStatus::Handled
            }
            MouseEvent::Up { button, .. } if button == MouseButton::LEFT => {
                self.is_mouse_selecting = false;
                ctx.release_capture();
                EventStatus::Handled
            }
            MouseEvent::Moved { position, .. } => {
                if self.is_mouse_selecting {
                    if let Some(new_cursor) = self
                        .text_layout
                        .text_index_at_point(position - ctx.bounds().top_left())
                    {
                        if self.set_caret_position(new_cursor, true) {
                            ctx.request_render();
                        }
                    }
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn animation_frame(&mut self, frame: crate::AnimationFrame, ctx: &mut AnimationContext) {
        if ctx.has_focus() {
            if frame.timestamp - self.last_cursor_timestamp > CURSOR_DELAY_SECONDS {
                self.cursor_on = !self.cursor_on;
                ctx.request_render();
                self.last_cursor_timestamp = frame.timestamp;
            }
            ctx.request_animation();
        }
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        match event {
            StatusChange::FocusGained => {
                ctx.request_animation();
                ctx.request_render();
            }
            StatusChange::FocusLost => {
                self.cursor_on = false;
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let bounds = ctx.global_bounds();

        let stroke_color = if ctx.has_focus() {
            Color::RED
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        };
        ctx.stroke(bounds.shrink(1.0), stroke_color, 1.0);

        let text_bounds = ctx.content_bounds();
        ctx.use_clip(text_bounds, |ctx| {
            if let Some(selection) = self.selection() {
                let left = self.text_layout.point_at_text_index(selection.start);
                let right = self.text_layout.point_at_text_index(selection.end);
                let rect = Rectangle::from_points(
                    text_bounds.top_left() + left,
                    text_bounds.bottom_left() + right,
                );
                ctx.fill(rect, Color::from_rgb8(68, 85, 90));
            }

            ctx.draw_text(&self.text_layout, text_bounds.origin());

            if ctx.has_focus() && self.cursor_on {
                let cursor_point = self.text_layout.point_at_text_index(self.position);
                let p0 = text_bounds.bottom_left() + cursor_point;
                let p1 = text_bounds.top_left() + cursor_point;
                ctx.draw_line(p0, p1, Color::BLACK, 1.0);
            }
        });
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }
}
