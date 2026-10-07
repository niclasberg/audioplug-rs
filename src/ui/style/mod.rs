mod display_style;
mod length;

use crate::{
    core::{Color, ColorMap, Cursor, Size, Zero},
    ui::render::ShadowOptions,
};
pub use display_style::{AvailableSpace, DisplayStyle, FlexStyle, GridStyle, Measure};
pub use length::{Corner, Corners, Edge, Edges, Length, RelativePoint};
pub use taffy::{
    AlignContent, AlignItems, AlignSelf, FlexDirection, FlexWrap, JustifyContent, JustifySelf,
    Overflow,
};

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
    pub padding: Edges,
    pub border: Option<Border>,
    pub margin: Edges,
    pub inset: Edges,
    pub scrollbar_width: f64,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub background: Option<Fill>,
    pub corner_radius: Size,
    pub cursor: Option<Cursor>,
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
            padding: Edges::ZERO,
            border: None,
            margin: Edges::ZERO,
            inset: Edges::ZERO,
            scrollbar_width: 5.0,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            background: None,
            corner_radius: Size::ZERO,
            cursor: None,
            align_self: None,
            justify_self: None,
            box_shadow: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Border {
    pub fill: Fill,
    pub width: Length,
}

impl Border {
    pub fn new(fill: impl Into<Fill>, width: Length) -> Self {
        Self {
            fill: fill.into(),
            width,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Fill {
    Solid(Color),
    LinearGradient {
        colors: ColorMap,
        start: RelativePoint,
        end: RelativePoint,
    },
    RadialGradient {
        colors: ColorMap,
        center: RelativePoint,
        radius: Length,
    },
}

impl Fill {}

impl From<Color> for Fill {
    fn from(value: Color) -> Self {
        Fill::Solid(value)
    }
}
