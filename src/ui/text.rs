use parley::{FontContext, FontStyle, FontWeight, FontWidth, Selection, StyleProperty};

use crate::{
    core::{Color, Rect, TextLayout, TextLayoutContext},
    ui::render::GlyphCache,
};

bitflags::bitflags! {
    struct TextFlags: u8 {
        const SELECTABLE = 1;
        const EDITABLE = 1 << 1;
    }
}

pub struct TextData {
    pub(super) layout_dirty: bool,
    pub(super) layout: TextLayout,
    pub text: String,
    pub font_size: f32,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub font_width: Option<FontWidth>,
    pub color: Color,
    pub selection: Selection,
    pub flags: TextFlags,
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
            self.selection = self.selection.refresh(&self.layout);
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
        !self.selection.is_collapsed()
    }

    pub fn cursor_rect(&self, width: f32) -> Rect {
        let bounds = self.selection.focus().geometry(&self.layout, width);
        Rect {
            left: bounds.x0 as _,
            top: bounds.y0 as _,
            right: bounds.x1 as _,
            bottom: bounds.y1 as _,
        }
    }

    pub fn with_selection_rects(&self, mut f: impl FnMut(Rect)) {
        self.selection.geometry_with(&self.layout, |bounds, _| {
            f(Rect {
                left: bounds.x0 as _,
                top: bounds.y0 as _,
                right: bounds.x1 as _,
                bottom: bounds.y1 as _,
            });
        });
    }

    pub fn selected_text(&self) -> Option<&str> {
        let range = self.selection.text_range();
        if range.is_empty() {
            None
        } else {
            Some(&self.text[range])
        }
    }

    fn set_selection(&mut self, new_selection: Selection) -> bool {
        if self.selection != new_selection {
            self.selection = new_selection;
            true
        } else {
            false
        }
    }

    pub fn move_right(&mut self, select: bool) -> bool {
        let selection = self.selection.next_visual(&self.layout, select);
        self.set_selection(selection)
    }

    pub fn move_word_right(&mut self, select: bool) -> bool {
        let selection = self.selection.next_visual_word(&self.layout, select);
        self.set_selection(selection)
    }

    pub fn move_left(&mut self, select: bool) -> bool {
        let selection = self.selection.previous_visual(&self.layout, select);
        self.set_selection(selection)
    }

    pub fn move_word_left(&mut self, select: bool) -> bool {
        let selection = self.selection.previous_visual_word(&self.layout, select);
        self.set_selection(selection)
    }

    fn remove_selection(&mut self) -> bool {
        if self.has_selection() {
            self.text.replace_range(self.selection.text_range(), "");
            self.layout_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn remove_left(&mut self) -> bool {
        self.remove_selection()
    }

    pub fn remove_right(&mut self) -> bool {
        self.remove_selection()
    }

    pub fn remove_word_left(&mut self) -> bool {
        self.remove_selection() || { false }
    }

    pub fn remove_word_right(&mut self) -> bool {
        false
    }
}

impl Default for TextData {
    fn default() -> Self {
        Self {
            layout: TextLayout::new(),
            text: "".to_string(),
            selection: Selection::default(),
            font_size: 12.0,
            font_weight: Default::default(),
            font_style: Default::default(),
            font_width: Default::default(),
            color: Color::BLACK,
            layout_dirty: true,
            flags: TextFlags::empty(),
        }
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
