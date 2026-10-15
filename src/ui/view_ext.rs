use taffy::{AlignSelf, JustifySelf};

use crate::{
    core::{Cursor, Size},
    ui::{
        Prop,
        reactive::{Computed, ReadContext},
        render::ShadowOptions,
        style::{Border, Edges, Fill, Length},
    },
};

pub struct ViewStyle {
    pub(crate) hidden: Option<Prop<bool>>,
    pub(crate) padding: Option<Prop<Edges>>,
    pub(crate) width: Option<Prop<Length>>,
    pub(crate) height: Option<Prop<Length>>,
    pub(crate) min_width: Option<Prop<Length>>,
    pub(crate) min_height: Option<Prop<Length>>,
    pub(crate) max_width: Option<Prop<Length>>,
    pub(crate) max_height: Option<Prop<Length>>,
    pub(crate) aspect_ratio: Option<Prop<f64>>,
    pub(crate) border: Option<Prop<Border>>,
    pub(crate) margin: Option<Prop<Edges>>,
    pub(crate) inset: Option<Prop<Edges>>,
    pub(crate) background: Option<Prop<Fill>>,
    pub(crate) corner_radius: Option<Prop<Size>>,
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
        justify_self: None,
        align_self: None,
        box_shadow: None,
        flex_grow: None,
        flex_shrink: None,
        cursor: None,
    };
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

    fn padding(mut self, value: impl Into<Prop<Edges>>) -> Self {
        self.style_mut().padding = Some(value.into());
        self
    }

    fn margin(mut self, value: impl Into<Prop<Edges>>) -> Self {
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

    fn background(mut self, value: impl Into<Prop<Fill>>) -> Self {
        self.style_mut().background = Some(value.into());
        self
    }

    fn corner_radius(mut self, value: impl Into<Prop<Size>>) -> Self {
        self.style_mut().corner_radius = Some(value.into());
        self
    }

    fn border(mut self, value: impl Into<Prop<Border>>) -> Self {
        self.style_mut().border = Some(value.into());
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

    fn shadow(mut self, value: impl Into<Prop<ShadowOptions>>) -> Self {
        self.style_mut().box_shadow = Some(value.into());
        self
    }

    fn cursor(mut self, value: impl Into<Prop<Cursor>>) -> Self {
        self.style_mut().cursor = Some(value.into());
        self
    }
}

impl StyleExt for ViewStyle {
    fn style_mut(&mut self) -> &mut ViewStyle {
        self
    }
}
