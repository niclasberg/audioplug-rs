use std::ops::Range;

use parley::{Cursor, FontContext, FontStyle, FontWeight, FontWidth, StyleProperty};

use crate::{
    core::{Color, TextLayout, TextLayoutContext},
    ui::render::GlyphCache,
};

pub struct TextData {
    pub(super) layout_dirty: bool,
    pub(super) layout: TextLayout,
    pub text: String,
    pub font_size: f32,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub font_width: Option<FontWidth>,
    pub color: Color,
    pub(crate) cursor: Cursor,
    pub(crate) selection_start: Cursor,
}

impl TextData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn rebuild_layout(&mut self, cx: &mut TextContext) {
        if self.layout_dirty {
            let mut builder = cx
                .layout_cx
                .ranged_builder(cx.font_cx, &self.text, 1.0, false);
            builder.push_default(StyleProperty::FontSize(self.font_size));
            if let Some(font_weight) = self.font_weight {
                builder.push_default(font_weight);
            }
            if let Some(font_style) = self.font_style {
                builder.push_default(font_style);
            }
            if let Some(font_width) = self.font_width {
                builder.push_default(font_width);
            }

            builder.build_into(&mut self.layout, &self.text);
            self.cursor = self.cursor.refresh(&self.layout);
            self.selection_start = self.selection_start.refresh(&self.layout);
            self.layout_dirty = false;
        }
    }

    pub fn measure(
        &mut self,
        available_space: taffy::Size<taffy::AvailableSpace>,
        known_dimensions: taffy::Size<Option<f32>>,
    ) -> taffy::Size<f32> {
        debug_assert!(!self.layout_dirty);
        let width = known_dimensions.width.unwrap_or_else(|| {
            let widths = self.layout.calculate_content_widths();
            match available_space.width {
                taffy::AvailableSpace::MinContent => widths.min,
                taffy::AvailableSpace::MaxContent => widths.max,
                taffy::AvailableSpace::Definite(limit) => limit.min(widths.max).max(widths.min),
            }
            .ceil()
        });

        self.layout.break_all_lines(Some(width));
        let height = known_dimensions
            .height
            .unwrap_or_else(|| self.layout.height());

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
}

impl Default for TextData {
    fn default() -> Self {
        Self {
            layout: TextLayout::new(),
            text: "".to_string(),
            cursor: Cursor::default(),
            selection_start: Cursor::default(),
            font_size: 12.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            font_width: Default::default(),
            color: Color::BLACK,
            layout_dirty: true,
        }
    }
}

pub struct TextEditContext<'a> {
    pub(super) text: &'a mut TextData,
}

impl TextEditContext<'_> {
    pub fn move_right(&mut self, select: bool) -> bool {
        let cursor = self.text.cursor.next_visual(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }

    pub fn move_word_right(&mut self, select: bool) -> bool {
        let cursor = self.text.cursor.next_visual_word(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }

    pub fn move_left(&mut self, select: bool) -> bool {
        let cursor = self.text.cursor.previous_visual(&self.text.layout);
        self.text.set_cursor(cursor, select)
    }

    pub fn move_word_left(&mut self, select: bool) -> bool {
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
