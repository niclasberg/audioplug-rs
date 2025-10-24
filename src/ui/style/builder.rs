use taffy::{AlignSelf, JustifySelf};

use crate::{
    core::{Brush, Color, ShadowOptions, Size},
    ui::{Accessor, BuildContext, Widget},
};

use super::{ImageEffect, Length, Style, UiRect};

#[derive(Default, Clone)]
pub struct StyleBuilder {
    hidden: Option<Accessor<bool>>,
    padding: Option<Accessor<UiRect>>,
    width: Option<Accessor<Length>>,
    height: Option<Accessor<Length>>,
    min_width: Option<Accessor<Length>>,
    min_height: Option<Accessor<Length>>,
    max_width: Option<Accessor<Length>>,
    max_height: Option<Accessor<Length>>,
    aspect_ratio: Option<Accessor<f64>>,
    border: Option<Accessor<Length>>,
    margin: Option<Accessor<UiRect>>,
    inset: Option<Accessor<UiRect>>,
    background: Option<Accessor<Brush>>,
    corner_radius: Option<Accessor<Size>>,
    border_color: Option<Accessor<Color>>,
    justify_self: Option<Accessor<JustifySelf>>,
    align_self: Option<Accessor<AlignSelf>>,
    box_shadow: Option<Accessor<ShadowOptions>>,
    effects: Option<Accessor<Vec<ImageEffect>>>,
    flex_grow: Option<Accessor<f32>>,
    flex_shrink: Option<Accessor<f32>>,
}

impl StyleBuilder {
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
        effects: None,
        flex_grow: None,
        flex_shrink: None,
    };

    pub fn hidden(&mut self, value: impl Into<Accessor<bool>>) -> &mut Self {
        self.hidden = Some(value.into());
        self
    }

    pub fn padding(&mut self, value: impl Into<Accessor<UiRect>>) -> &mut Self {
        self.padding = Some(value.into());
        self
    }

    pub fn margin(&mut self, value: impl Into<Accessor<UiRect>>) -> &mut Self {
        self.margin = Some(value.into());
        self
    }

    pub fn height(&mut self, value: impl Into<Accessor<Length>>) -> &mut Self {
        self.height = Some(value.into());
        self
    }

    pub fn width(&mut self, value: impl Into<Accessor<Length>>) -> &mut Self {
        self.width = Some(value.into());
        self
    }

    pub fn min_width(&mut self, value: impl Into<Accessor<Length>>) -> &mut Self {
        self.min_width = Some(value.into());
        self
    }

    pub fn max_width(&mut self, value: impl Into<Accessor<Length>>) -> &mut Self {
        self.max_width = Some(value.into());
        self
    }

    pub fn min_height(&mut self, value: impl Into<Accessor<Length>>) -> &mut Self {
        self.min_height = Some(value.into());
        self
    }

    pub fn max_height(&mut self, value: impl Into<Accessor<Length>>) -> &mut Self {
        self.max_height = Some(value.into());
        self
    }

    pub fn background(&mut self, value: impl Into<Accessor<Brush>>) -> &mut Self {
        self.background = Some(value.into());
        self
    }

    pub fn corner_radius(&mut self, value: impl Into<Accessor<Size>>) -> &mut Self {
        self.corner_radius = Some(value.into());
        self
    }

    pub fn border(
        &mut self,
        value: impl Into<Accessor<Length>>,
        color: impl Into<Accessor<Color>>,
    ) -> &mut Self {
        self.border = Some(value.into());
        self.border_color = Some(color.into());
        self
    }

    pub fn align_self(&mut self, value: impl Into<Accessor<AlignSelf>>) -> &mut Self {
        self.align_self = Some(value.into());
        self
    }

    pub fn flex_grow(&mut self, value: impl Into<Accessor<f32>>) -> &mut Self {
        self.flex_grow = Some(value.into());
        self
    }

    pub fn flex_shrink(&mut self, value: impl Into<Accessor<f32>>) -> &mut Self {
        self.flex_shrink = Some(value.into());
        self
    }

    pub fn box_shadow(&mut self, value: impl Into<Accessor<ShadowOptions>>) -> &mut Self {
        self.box_shadow = Some(value.into());
        self
    }

    pub fn effects(&mut self, value: impl Into<Accessor<Vec<ImageEffect>>>) -> &mut Self {
        self.effects = Some(value.into());
        self
    }

    pub(crate) fn merge(&mut self, other: Self) {
        replace_if_some(&mut self.hidden, other.hidden);
        replace_if_some(&mut self.padding, other.padding);
        replace_if_some(&mut self.width, other.width);
        replace_if_some(&mut self.height, other.height);
        replace_if_some(&mut self.aspect_ratio, other.aspect_ratio);
        replace_if_some(&mut self.border, other.border);
        replace_if_some(&mut self.margin, other.margin);
        replace_if_some(&mut self.inset, other.inset);
        replace_if_some(&mut self.min_width, other.min_width);
        replace_if_some(&mut self.max_width, other.max_width);
        replace_if_some(&mut self.max_height, other.max_height);
        replace_if_some(&mut self.background, other.background);
        replace_if_some(&mut self.corner_radius, other.corner_radius);
        replace_if_some(&mut self.border_color, other.border_color);
        replace_if_some(&mut self.justify_self, other.justify_self);
        replace_if_some(&mut self.align_self, other.align_self);
        replace_if_some(&mut self.box_shadow, other.box_shadow);
        replace_if_some(&mut self.flex_grow, other.flex_grow);
        replace_if_some(&mut self.flex_shrink, other.flex_shrink);
    }

    pub(crate) fn apply_styles(self, cx: &mut BuildContext<dyn Widget>) {
        apply_layout_style(self.aspect_ratio, cx, |value, style| {
            style.aspect_ratio = Some(value);
        });
        apply_render_style(self.background, cx, |value, style| {
            style.background = Some(value);
        });
        apply_layout_style(self.border, cx, |value, style| {
            style.border = value;
        });
        apply_layout_style(self.corner_radius, cx, |value, style| {
            style.corner_radius = value;
        });
        apply_layout_style(self.height, cx, |value, style| {
            style.size.height = value;
        });
        apply_layout_style(self.hidden, cx, |value, style| {
            style.hidden = value;
        });
        apply_layout_style(self.min_height, cx, |value, style| {
            style.min_size.height = value;
        });
        apply_layout_style(self.min_width, cx, |value, style| {
            style.min_size.width = value;
        });
        apply_layout_style(self.max_height, cx, |value, style| {
            style.max_size.height = value;
        });
        apply_layout_style(self.max_width, cx, |value, style| {
            style.max_size.width = value;
        });
        apply_layout_style(self.padding, cx, |value, style| {
            style.padding = value;
        });
        apply_layout_style(self.width, cx, |value, style| {
            style.size.width = value;
        });
        apply_render_style(self.border_color, cx, |value, style| {
            style.border_color = Some(value);
        });
        apply_layout_style(self.align_self, cx, |value, style| {
            style.align_self = Some(value);
        });
        apply_layout_style(self.flex_grow, cx, |value, style| style.flex_grow = value);
        apply_layout_style(self.flex_shrink, cx, |value, style| {
            style.flex_shrink = value
        });
        apply_render_style(self.box_shadow, cx, |value, style| {
            style.box_shadow = Some(value);
        })
    }
}

fn replace_if_some<T>(current: &mut Option<T>, new_value: Option<T>) {
    if let Some(new_value) = new_value {
        current.replace(new_value);
    }
}

fn apply_layout_style<T: Clone + 'static>(
    accessor: Option<Accessor<T>>,
    ctx: &mut BuildContext<dyn Widget>,
    apply_fn: fn(T, &mut Style),
) {
    if let Some(accessor) = accessor {
        let value = accessor.get_and_bind(ctx, move |value, mut widget| {
            widget.update_style(|style| apply_fn(value, style));
            widget.request_layout();
        });
        ctx.update_default_style(|style| apply_fn(value, style));
    }
}

fn apply_render_style<T: Clone + 'static>(
    accessor: Option<Accessor<T>>,
    ctx: &mut BuildContext<dyn Widget>,
    apply_fn: fn(T, &mut Style),
) {
    if let Some(accessor) = accessor {
        let value = accessor.get_and_bind(ctx, move |value, mut widget| {
            widget.update_style(|style| apply_fn(value, style));
            widget.request_render();
        });
        ctx.update_default_style(|style| apply_fn(value, style));
    }
}
