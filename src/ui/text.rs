use std::cell::RefCell;

use parley::{
    Affinity, Cursor, FontContext, FontStyle, FontWeight, FontWidth, Selection, StyleProperty,
};

use crate::core::{Color, Rect, TextLayout, TextLayoutContext};

thread_local! {
    static TEXT_CX: RefCell<(FontContext, TextLayoutContext)> = RefCell::new(Default::default());
}

pub(crate) fn with_text_cx<R>(f: impl FnOnce(&mut FontContext, &mut TextLayoutContext) -> R) -> R {
    TEXT_CX.with_borrow_mut(|(font, layout)| f(font, layout))
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
    width: Option<f32>,
    pub is_editable: bool,
}

impl TextData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn rebuild_layout(&mut self) {
        if self.layout_dirty {
            self.force_rebuild_layout();
        }
    }

    fn force_rebuild_layout(&mut self) {
        with_text_cx(|font_cx, layout_cx| {
            let mut builder = layout_cx.ranged_builder(font_cx, &self.text, 1.0, false);
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
        });
        self.layout.break_all_lines(self.width);
        self.selection = self.selection.refresh(&self.layout);
        self.layout_dirty = false;
    }

    pub fn measure(
        &mut self,
        available_space: taffy::Size<taffy::AvailableSpace>,
        known_dimensions: taffy::Size<Option<f32>>,
    ) -> taffy::Size<f32> {
        self.rebuild_layout();
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
        self.width = Some(width);
        let height = known_dimensions
            .height
            .unwrap_or_else(|| self.layout.height());

        taffy::Size { width, height }
    }

    pub fn is_editable(&self) -> bool {
        self.is_editable
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
        self.rebuild_layout();
        let selection = self.selection.next_visual(&self.layout, select);
        self.set_selection(selection)
    }

    pub fn move_word_right(&mut self, select: bool) -> bool {
        self.rebuild_layout();
        let selection = self.selection.next_visual_word(&self.layout, select);
        self.set_selection(selection)
    }

    pub fn move_left(&mut self, select: bool) -> bool {
        self.rebuild_layout();
        let selection = self.selection.previous_visual(&self.layout, select);
        self.set_selection(selection)
    }

    pub fn move_word_left(&mut self, select: bool) -> bool {
        self.rebuild_layout();
        let selection = self.selection.previous_visual_word(&self.layout, select);
        self.set_selection(selection)
    }

    /// Remove glyph to the left of the cursor (backspace)
    pub fn remove_left(&mut self) -> bool {
        self.rebuild_layout();
        if self.selection.is_collapsed() {
            let Some(cluster) = self.selection.focus().logical_clusters(&self.layout)[0] else {
                return false;
            };

            let range = cluster.text_range();
            if range.is_empty() {
                false
            } else {
                self.text.replace_range(range.clone(), "");
                self.force_rebuild_layout();
                self.selection =
                    Cursor::from_byte_index(&self.layout, range.start, Affinity::Downstream).into();
                true
            }
        } else {
            self.insert_or_replace("")
        }
    }

    pub fn remove_word_left(&mut self) -> bool {
        self.rebuild_layout();
        if self.selection.is_collapsed() {
            let focus = self.selection.focus();
            let end = focus.index();
            let start = focus.previous_logical_word(&self.layout).index();
            let range_to_remove = start..end;
            if range_to_remove.is_empty() {
                false
            } else {
                self.text.replace_range(range_to_remove, "");
                self.force_rebuild_layout();
                self.selection =
                    Cursor::from_byte_index(&self.layout, start, Affinity::Downstream).into();
                true
            }
        } else {
            self.insert_or_replace("")
        }
    }

    pub fn remove_right(&mut self) -> bool {
        self.rebuild_layout();
        if self.selection.is_collapsed() {
            let Some(cluster) = self.selection.focus().logical_clusters(&self.layout)[1] else {
                return false;
            };

            let range = cluster.text_range();
            if range.is_empty() {
                false
            } else {
                self.text.replace_range(range.clone(), "");
                self.force_rebuild_layout();
                true
            }
        } else {
            self.insert_or_replace("")
        }
    }

    pub fn remove_word_right(&mut self) -> bool {
        self.rebuild_layout();
        if self.selection.is_collapsed() {
            let focus = self.selection.focus();
            let start = focus.index();
            let end = focus.next_logical_word(&self.layout).index();
            let range_to_remove = start..end;
            if range_to_remove.is_empty() {
                false
            } else {
                self.text.replace_range(range_to_remove, "");
                self.force_rebuild_layout();
                self.selection =
                    Cursor::from_byte_index(&self.layout, start, Affinity::Downstream).into();
                true
            }
        } else {
            self.insert_or_replace("")
        }
    }

    pub fn insert_or_replace(&mut self, str: &str) -> bool {
        let selected_range = self.selection.text_range();
        if selected_range.is_empty() {
            if str.is_empty() {
                return false;
            }
            self.text.insert_str(selected_range.start, str);
        } else {
            self.text.replace_range(selected_range.clone(), str);
        }
        self.force_rebuild_layout();

        let new_index = selected_range.start.saturating_add(str.len());
        let affinity = if str.ends_with(['\n', '\r', '\u{2028}', '\u{2029}']) {
            Affinity::Downstream
        } else {
            Affinity::Upstream
        };
        self.selection = Cursor::from_byte_index(&self.layout, new_index, affinity).into();
        true
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
            is_editable: false,
            width: None,
        }
    }
}
