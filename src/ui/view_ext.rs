use taffy::{AlignSelf, JustifySelf};

use crate::{
    core::{Color, Cursor, Paint, ShadowOptions, Size},
    ui::{
        Prop,
        reactive::{Computed, ReadContext},
        style::{Length, UiRect},
    },
};

pub struct ViewStyle {
    pub(crate) hidden: Option<Prop<bool>>,
    pub(crate) padding: Option<Prop<UiRect>>,
    pub(crate) width: Option<Prop<Length>>,
    pub(crate) height: Option<Prop<Length>>,
    pub(crate) min_width: Option<Prop<Length>>,
    pub(crate) min_height: Option<Prop<Length>>,
    pub(crate) max_width: Option<Prop<Length>>,
    pub(crate) max_height: Option<Prop<Length>>,
    pub(crate) aspect_ratio: Option<Prop<f64>>,
    pub(crate) border: Option<Prop<Length>>,
    pub(crate) margin: Option<Prop<UiRect>>,
    pub(crate) inset: Option<Prop<UiRect>>,
    pub(crate) background: Option<Prop<Paint>>,
    pub(crate) corner_radius: Option<Prop<Size>>,
    pub(crate) border_color: Option<Prop<Color>>,
    pub(crate) justify_self: Option<Prop<JustifySelf>>,
    pub(crate) align_self: Option<Prop<AlignSelf>>,
    pub(crate) box_shadow: Option<Prop<ShadowOptions>>,
    pub(crate) flex_grow: Option<Prop<f32>>,
    pub(crate) flex_shrink: Option<Prop<f32>>,
    pub(crate) cursor: Option<Prop<Cursor>>,
}

impl ViewStyle {
    pub const DEFAULT: Self = Self {
        hidden: None,
        padding: None,
        width: None,
        height: None,
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        aspect_ratio: None,
        border: None,
        margin: None,
        inset: None,
        background: None,
        corner_radius: None,
        border_color: None,
        justify_self: None,
        align_self: None,
        box_shadow: None,
        flex_grow: None,
        flex_shrink: None,
        cursor: None,
    };

    pub fn hidden(mut self, value: bool) -> Self {
        self.hidden.replace(Prop::Const(value));
        self
    }

    pub fn padding(mut self, value: UiRect) -> Self {
        self.padding.replace(Prop::Const(value));
        self
    }

    pub fn background(mut self, value: Paint) -> Self {
        self.background.replace(value.into());
        self
    }

    pub fn background_fn(mut self, f: fn(&mut ReadContext) -> Paint) -> Self {
        self.background.replace(Computed::new(f).into());
        self
    }

    pub fn margin(mut self, value: UiRect) -> Self {
        self.margin.replace(Prop::Const(value));
        self
    }

    pub fn height(mut self, value: Length) -> Self {
        self.height.replace(value.into());
        self
    }

    pub fn width(mut self, value: Length) -> Self {
        self.width.replace(value.into());
        self
    }

    pub fn min_width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.min_width.replace(value.into());
        self
    }

    pub fn max_width(mut self, value: Length) -> Self {
        self.max_width.replace(value.into());
        self
    }

    pub fn min_height(mut self, value: Length) -> Self {
        self.min_height.replace(value.into());
        self
    }

    pub fn max_height(mut self, value: Length) -> Self {
        self.max_height.replace(value.into());
        self
    }

    pub fn corner_radius(mut self, value: Size) -> Self {
        self.corner_radius.replace(Prop::Const(value));
        self
    }

    pub fn border_width(mut self, value: Length) -> Self {
        self.border.replace(Prop::Const(value));
        self
    }

    pub fn border_color(mut self, value: Color) -> Self {
        self.border_color.replace(Prop::Const(value));
        self
    }

    pub fn align_self(mut self, value: impl Into<Prop<AlignSelf>>) -> Self {
        self.align_self.replace(value.into());
        self
    }

    pub fn flex_grow(mut self, value: f32) -> Self {
        self.flex_grow.replace(value.into());
        self
    }

    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.flex_shrink.replace(value.into());
        self
    }

    pub fn shadow(mut self, value: ShadowOptions) -> Self {
        self.box_shadow = Some(Prop::Const(value));
        self
    }

    pub fn cursor(mut self, value: Cursor) -> Self {
        self.cursor = Some(Prop::Const(value));
        self
    }
}

impl Default for ViewStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub trait StyleExt: Sized {
    fn style_mut(&mut self) -> &mut ViewStyle;

    fn style(&mut self, style: ViewStyle) {
        *self.style_mut() = style;
    }

    fn hidden(mut self, value: impl Into<Prop<bool>>) -> Self {
        self.style_mut().hidden = Some(value.into());
        self
    }

    fn padding(mut self, value: impl Into<Prop<UiRect>>) -> Self {
        self.style_mut().padding = Some(value.into());
        self
    }

    fn margin(mut self, value: impl Into<Prop<UiRect>>) -> Self {
        self.style_mut().margin = Some(value.into());
        self
    }

    fn height(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.style_mut().height = Some(value.into());
        self
    }

    fn width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.style_mut().width = Some(value.into());
        self
    }

    fn min_width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.style_mut().min_width = Some(value.into());
        self
    }

    fn max_width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.style_mut().max_width = Some(value.into());
        self
    }

    fn min_height(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.style_mut().min_height = Some(value.into());
        self
    }

    fn max_height(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.style_mut().max_height = Some(value.into());
        self
    }

    fn background(mut self, value: impl Into<Prop<Paint>>) -> Self {
        self.style_mut().background = Some(value.into());
        self
    }

    fn corner_radius(mut self, value: impl Into<Prop<Size>>) -> Self {
        self.style_mut().corner_radius = Some(value.into());
        self
    }

    fn border(mut self, value: impl Into<Prop<Length>>, color: impl Into<Prop<Color>>) -> Self {
        self.style_mut().border = Some(value.into());
        self.style_mut().border_color = Some(color.into());
        self
    }

    fn align_self(mut self, value: impl Into<Prop<AlignSelf>>) -> Self {
        self.style_mut().align_self = Some(value.into());
        self
    }

    fn flex_grow(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.style_mut().flex_grow = Some(value.into());
        self
    }

    fn flex_shrink(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.style_mut().flex_shrink = Some(value.into());
        self
    }

    fn box_shadow(mut self, value: impl Into<Prop<ShadowOptions>>) -> Self {
        self.style_mut().box_shadow = Some(value.into());
        self
    }
}
