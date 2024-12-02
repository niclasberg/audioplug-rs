mod display_style;
mod layout_style;
mod length;
mod style_builder;
mod ui_rect;

use crate::core::{Color, Size};
pub use display_style::{DisplayStyle, FlexStyle, Measure};
pub(crate) use layout_style::LayoutStyle;
pub use length::Length;
pub use style_builder::StyleBuilder;
pub use taffy::{FlexDirection, FlexWrap, Overflow};
pub use ui_rect::UiRect;

#[derive(Debug, Copy, Clone)]
pub struct Style {
    pub hidden: bool,
    pub size: Size<Length>,
    pub min_size: Size<Length>,
    pub max_size: Size<Length>,
    pub aspect_ratio: Option<f64>,
    pub padding: UiRect,
    pub border: UiRect,
    pub margin: UiRect,
    pub inset: UiRect,
    pub scrollbar_width: f64,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub background: Option<Color>,
    pub corner_radius: Size,
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
            border: UiRect::ZERO,
            margin: UiRect::ZERO,
            inset: UiRect::ZERO,
            scrollbar_width: 5.0,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            background: None,
            corner_radius: Size::ZERO,
        }
    }
}
