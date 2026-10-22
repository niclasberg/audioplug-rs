use std::ops::Range;

use parley::{Cursor, FontContext, FontStyle, FontWeight, FontWidth, RangedBuilder, StyleProperty};

use crate::{
    core::{Color, TextBrush, TextLayout, TextLayoutContext},
    ui::render::GlyphCache,
};

#[derive(Default)]
pub struct TextData {
    pub(crate) text: String,
    pub(crate) style: TextStyle,

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

    /// Set the text, returns true if the text changed
    pub fn set_text(&mut self, value: String) -> bool {
        if self.text != value {
            self.text = value;
            self.layout_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn set_font_size(&mut self, value: f32) -> bool {
        self.replace_and_invalidate(|this| &mut this.style.font_size, value)
    }

    fn replace_and_invalidate<T: PartialEq>(
        &mut self,
        dst_fn: fn(&mut Self) -> &mut T,
        new_val: T,
    ) -> bool {
        let dst = dst_fn(self);
        if *dst != new_val {
            *dst = new_val;
            self.layout_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn set_font_weight(&mut self, value: FontWeight) -> bool {
        self.replace_and_invalidate(|this| &mut this.style.font_weight, Some(value))
    }

    pub fn set_font_style(&mut self, value: FontStyle) -> bool {
        self.replace_and_invalidate(|this| &mut this.style.font_style, Some(value))
    }

    pub fn set_font_width(&mut self, value: FontWidth) -> bool {
        self.replace_and_invalidate(|this| &mut this.style.font_width, Some(value))
    }

    fn refresh_layout(&mut self, cx: &mut TextContext) {
        if self.layout_dirty {
            let mut builder = cx
                .layout_cx
                .ranged_builder(cx.font_cx, &self.text, 1.0, false);
            self.style.apply(&mut builder);
            builder.build_into(&mut self.layout, &self.text);
            self.cursor = self.cursor.refresh(&self.layout);
            self.selection_start = self.selection_start.refresh(&self.layout);
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
    pub color: Color,
}

impl TextStyle {
    pub fn new() -> Self {
        Self {
            font_size: 12.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            font_width: Default::default(),
            color: Color::BLACK,
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
