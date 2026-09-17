mod display_style;
mod image_effect;
mod length;
mod ui_rect;

use crate::core::{Color, Cursor, Paint, ShadowOptions, Size, Zero};
pub use display_style::{AvailableSpace, DisplayStyle, FlexStyle, GridStyle, LayoutMode, Measure};
pub use image_effect::ImageEffect;
pub use length::Length;
pub use taffy::{
    AlignContent, AlignItems, AlignSelf, FlexDirection, FlexWrap, JustifyContent, JustifySelf,
    Overflow,
};
pub use ui_rect::UiRect;

pub(crate) trait ResolveInto<T> {
    fn resolve_into(self, window_size: Size) -> T;
}

#[derive(Debug, Clone)]
pub struct Style {
    pub hidden: bool,
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub aspect_ratio: Option<f64>,
    pub padding: UiRect,
    pub border: Length,
    pub margin: UiRect,
    pub inset: UiRect,
    pub scrollbar_width: f64,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub background: Option<Paint>,
    pub corner_radius: Size,
    pub cursor: Option<Cursor>,
    pub border_color: Option<Color>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_self: Option<AlignSelf>,
    pub justify_self: Option<JustifySelf>,
    pub box_shadow: Option<ShadowOptions>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            hidden: false,
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            aspect_ratio: None,
            padding: UiRect::ZERO,
            border: Length::ZERO,
            margin: UiRect::ZERO,
            inset: UiRect::ZERO,
            scrollbar_width: 5.0,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            background: None,
            corner_radius: Size::ZERO,
            cursor: None,
            border_color: None,
            align_self: None,
            justify_self: None,
            box_shadow: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
        }
    }
}
