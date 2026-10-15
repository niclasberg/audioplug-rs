use std::ops::Range;

use parley::{Cursor, FontContext, FontStyle, FontWeight, FontWidth, RangedBuilder, StyleProperty};

use crate::{
    core::{Color, TextBrush, TextLayout, TextLayoutContext},
    ui::{Prop, render::GlyphCache},
};

pub struct TextData {
    text: String,
    style: TextStyle,

    layout: TextLayout,
    layout_dirty: bool,

    pub(crate) cursor: Cursor,
    pub(crate) selection_start: Cursor,
}

impl TextData {
    pub fn new() -> Self {
        Self {
            text: Default::default(),
            style: TextStyle::default(),
            layout: TextLayout::new(),
            layout_dirty: true,
            cursor: Cursor::default(),
            selection_start: Cursor::default(),
        }
    }

    pub fn set_text(&mut self, value: String) {
        if self.text != value {
            self.text = value;
            self.layout_dirty = true;
        }
    }

    pub fn set_font_size(&mut self, value: f32) {
        if self.style.font_size != value {
            self.style.font_size = value;
            self.layout_dirty = true;
        }
    }

    fn refresh_layout(&mut self, cx: &mut TextContext) {
        if self.layout_dirty {
            let mut builder = cx
                .layout_cx
                .ranged_builder(cx.font_cx, &self.text, 1.0, false);
            self.style.apply(&mut builder);
            builder.build_into(&mut self.layout, &self.text);
            self.cursor.refresh(&self.layout);
            self.selection_start.refresh(&self.layout);
            self.layout_dirty = false;
        }
    }

    pub fn get_or_create_layout(&mut self, cx: &mut TextContext) -> &mut TextLayout {
        self.refresh_layout(cx);
        &mut self.layout
    }

    pub fn measure(
        &mut self,
        cx: &mut TextContext,
        available_space: taffy::Size<taffy::AvailableSpace>,
        known_dimensions: taffy::Size<Option<f32>>,
    ) -> taffy::Size<f32> {
        let text_layout = self.get_or_create_layout(cx);
        let width = known_dimensions.width.unwrap_or_else(|| {
            let widths = text_layout.calculate_content_widths();
            match available_space.width {
                taffy::AvailableSpace::MinContent => widths.min,
                taffy::AvailableSpace::MaxContent => widths.max,
                taffy::AvailableSpace::Definite(limit) => limit.min(widths.max).max(widths.min),
            }
            .ceil()
        });

        text_layout.break_all_lines(Some(width));
        let height = known_dimensions
            .height
            .unwrap_or_else(|| text_layout.height());

        taffy::Size { width, height }
    }

    pub fn has_selection(&self) -> bool {
        self.cursor == self.selection_start
    }

    fn selected_range(&self) -> Range<usize> {
        let start = self.cursor.index().min(self.selection_start.index());
        let end = self.cursor.index().max(self.selection_start.index());
        start..end
    }

    pub fn selected_text(&self) -> Option<&str> {
        let range = self.selected_range();
        if !range.is_empty() {
            Some(&self.text[range])
        } else {
            None
        }
    }

    fn set_selection(&mut self, cursor: Cursor, selection_start: Cursor) -> bool {
        if self.cursor != cursor || selection_start != self.selection_start {
            self.cursor = cursor;
            self.selection_start = selection_start;
            true
        } else {
            false
        }
    }

    fn set_cursor(&mut self, cursor: Cursor, select: bool) -> bool {
        let selection_start = if select { self.selection_start } else { cursor };
        self.set_selection(cursor, selection_start)
    }

    fn remove_selected(&mut self) -> bool {
        let range = self.selected_range();
        if !range.is_empty() {
            self.text.replace_range(range, "");
            true
        } else {
            false
        }
    }
}

pub struct TextEditContext<'a> {
    text: &'a mut TextData,
    text_cx: TextContext<'a>,
}

impl TextEditContext<'_> {
    fn refresh_layout(&mut self) {
        self.text.refresh_layout(&mut self.text_cx);
    }

    pub fn move_right(&mut self, select: bool) -> bool {
        self.refresh_layout();
        let cursor = self.text.cursor.next_visual(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }

    pub fn move_word_right(&mut self, select: bool) -> bool {
        self.refresh_layout();
        let cursor = self.text.cursor.next_visual_word(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }

    pub fn move_left(&mut self, select: bool) -> bool {
        self.refresh_layout();
        let cursor = self.text.cursor.previous_visual(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }

    pub fn move_word_left(&mut self, select: bool) -> bool {
        self.refresh_layout();
        let cursor = self.text.cursor.previous_visual_word(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }
}

pub struct TextContext<'a> {
    pub font_cx: &'a mut FontContext,
    pub layout_cx: &'a mut TextLayoutContext,
    pub glyph_cache: &'a mut GlyphCache,
}

impl<'a> TextContext<'a> {
    pub fn new(
        font_cx: &'a mut FontContext,
        layout_cx: &'a mut TextLayoutContext,
        glyph_cache: &'a mut GlyphCache,
    ) -> Self {
        Self {
            font_cx,
            layout_cx,
            glyph_cache,
        }
    }
}

pub struct TextStyle {
    pub font_size: f32,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub font_width: Option<FontWidth>,
}

impl TextStyle {
    pub fn new() -> Self {
        Self {
            font_size: 12.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            font_width: Default::default(),
        }
    }

    fn apply(&self, range_builder: &mut RangedBuilder<'_, TextBrush>) {
        range_builder.push_default(StyleProperty::FontSize(self.font_size));
        if let Some(font_weight) = self.font_weight {
            range_builder.push_default(font_weight);
        }
        if let Some(font_style) = self.font_style {
            range_builder.push_default(font_style);
        }
        if let Some(font_width) = self.font_width {
            range_builder.push_default(font_width);
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct ViewText {
    pub(crate) text: Option<Prop<String>>,
    pub(crate) color: Option<Prop<Color>>,
    pub(crate) font_size: Option<Prop<f32>>,
}

impl ViewText {
    pub fn new(text: impl Into<Prop<String>>) -> Self {
        Self {
            text: Some(text.into()),
            color: None,
            font_size: None,
        }
    }
}

pub trait TextExt: Sized {
    fn text_mut(&mut self) -> &mut ViewText;

    fn text(mut self, value: impl Into<Prop<String>>) -> Self {
        self.text_mut().text.replace(value.into());
        self
    }

    fn font_size(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.text_mut().font_size.replace(value.into());
        self
    }

    fn color(mut self, value: impl Into<Prop<Color>>) -> Self {
        self.text_mut().color.replace(value.into());
        self
    }
}
