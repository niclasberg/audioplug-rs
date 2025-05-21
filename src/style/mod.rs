mod builder;
mod display_style;
mod image_effect;
mod length;
mod ui_rect;

use crate::{
    app::Brush,
    core::{Color, Cursor, ShadowOptions, Size},
};
pub use builder::StyleBuilder;
pub use display_style::{AvailableSpace, DisplayStyle, FlexStyle, GridStyle, Measure};
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

impl<U, T: ResolveInto<U>> ResolveInto<taffy::Size<U>> for Size<T> {
    fn resolve_into(self, window_size: Size) -> taffy::Size<U> {
        taffy::Size {
            width: self.width.resolve_into(window_size),
            height: self.height.resolve_into(window_size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub hidden: bool,
    pub size: Size<Length>,
    pub min_size: Size<Length>,
    pub max_size: Size<Length>,
    pub aspect_ratio: Option<f64>,
    pub padding: UiRect,
    pub border: Length,
    pub margin: UiRect,
    pub inset: UiRect,
    pub scrollbar_width: f64,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub background: Option<Brush>,
    pub corner_radius: Size,
    pub cursor: Option<Cursor>,
    pub border_color: Option<Color>,
    pub align_self: Option<AlignSelf>,
    pub justify_self: Option<JustifySelf>,
    pub box_shadow: Option<ShadowOptions>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            hidden: false,
            size: Default::default(),
            min_size: Default::default(),
            max_size: Default::default(),
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
        }
    }
}
