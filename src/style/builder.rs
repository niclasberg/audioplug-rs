use taffy::{AlignSelf, JustifySelf};

use crate::{
    app::{Accessor, Brush, BuildContext, Widget},
    core::{Color, Size},
};

use super::{Length, Style, UiRect};

#[derive(Default)]
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
}

impl StyleBuilder {
    pub fn hidden(mut self, value: impl Into<Accessor<bool>>) -> Self {
        self.hidden = Some(value.into());
        self
    }

    pub fn padding(mut self, value: impl Into<Accessor<UiRect>>) -> Self {
        self.padding = Some(value.into());
        self
    }

    pub fn margin(mut self, value: impl Into<Accessor<UiRect>>) -> Self {
        self.margin = Some(value.into());
        self
    }

    pub fn height(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.height = Some(value.into());
        self
    }

    pub fn width(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.width = Some(value.into());
        self
    }

    pub fn min_width(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.min_width = Some(value.into());
        self
    }

    pub fn max_width(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.max_width = Some(value.into());
        self
    }

    pub fn min_height(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.min_height = Some(value.into());
        self
    }

    pub fn max_height(mut self, value: impl Into<Accessor<Length>>) -> Self {
        self.max_height = Some(value.into());
        self
    }

    pub fn background(mut self, value: impl Into<Accessor<Brush>>) -> Self {
        self.background = Some(value.into());
        self
    }

    pub fn corner_radius(mut self, value: impl Into<Accessor<Size>>) -> Self {
        self.corner_radius = Some(value.into());
        self
    }

    pub fn border(
        mut self,
        value: impl Into<Accessor<Length>>,
        color: impl Into<Accessor<Color>>,
    ) -> Self {
        self.border = Some(value.into());
        self.border_color = Some(color.into());
        self
    }

    pub fn align_self(mut self, value: impl Into<Accessor<AlignSelf>>) -> Self {
        self.align_self = Some(value.into());
        self
    }

    pub fn justify_self(mut self, value: impl Into<Accessor<JustifySelf>>) -> Self {
        self.justify_self = Some(value.into());
        self
    }

    pub(crate) fn merge(&mut self, other: Self) {
        if let Some(hidden) = other.hidden {
            self.hidden.replace(hidden);
        }
        if let Some(padding) = other.padding {
            self.padding.replace(padding);
        }
        if let Some(width) = other.width {
            self.width.replace(width);
        }
        if let Some(height) = other.height {
            self.height.replace(height);
        }
        if let Some(aspect_ratio) = other.aspect_ratio {
            self.aspect_ratio.replace(aspect_ratio);
        }
        if let Some(border) = other.border {
            self.border.replace(border);
        }
        if let Some(margin) = other.margin {
            self.margin.replace(margin);
        }
        if let Some(inset) = other.inset {
            self.inset.replace(inset);
        }
        if let Some(min_width) = other.min_width {
            self.min_width.replace(min_width);
        }
        if let Some(min_height) = other.min_height {
            self.min_height.replace(min_height);
        }
        if let Some(max_width) = other.max_width {
            self.max_width.replace(max_width);
        }
        if let Some(max_height) = other.max_height {
            self.max_height.replace(max_height);
        }
        if let Some(background) = other.background {
            self.background.replace(background);
        }
        if let Some(corner_radius) = other.corner_radius {
            self.corner_radius.replace(corner_radius);
        }
        if let Some(border_color) = other.border_color {
            self.border_color.replace(border_color);
        }
        if let Some(justify_self) = other.justify_self {
            self.justify_self.replace(justify_self);
        }
        if let Some(align_self) = other.align_self {
            self.align_self.replace(align_self);
        }
    }
}

pub fn apply_styles<W: Widget>(ctx: &mut BuildContext<W>, style_builder: StyleBuilder) {
    apply_style(style_builder.aspect_ratio, ctx, |value, style| {
        style.aspect_ratio = Some(value);
    });
    apply_style(style_builder.background, ctx, |value, style| {
        style.background = Some(value);
    });
    apply_style(style_builder.border, ctx, |value, style| {
        style.border = value;
    });
    apply_style(style_builder.corner_radius, ctx, |value, style| {
        style.corner_radius = value;
    });
    apply_style(style_builder.height, ctx, |value, style| {
        style.size.height = value;
    });
    apply_style(style_builder.hidden, ctx, |value, style| {
        style.hidden = value;
    });
    apply_style(style_builder.min_height, ctx, |value, style| {
        style.min_size.height = value;
    });
    apply_style(style_builder.min_width, ctx, |value, style| {
        style.min_size.width = value;
    });
    apply_style(style_builder.max_height, ctx, |value, style| {
        style.max_size.height = value;
    });
    apply_style(style_builder.max_width, ctx, |value, style| {
        style.max_size.width = value;
    });
    apply_style(style_builder.padding, ctx, |value, style| {
        style.padding = value;
    });
    apply_style(style_builder.width, ctx, |value, style| {
        style.size.width = value;
    });
    apply_style(style_builder.border_color, ctx, |value, style| {
        style.border_color = Some(value);
    });
    apply_style(style_builder.align_self, ctx, |value, style| {
        style.align_self = Some(value);
    });
}

fn apply_style<W: Widget, T: Clone + 'static>(
    accessor: Option<Accessor<T>>,
    ctx: &mut BuildContext<W>,
    apply_fn: impl Fn(T, &mut Style) + 'static + Copy,
) {
    if let Some(accessor) = accessor {
        let value = accessor.get_and_bind(ctx, move |value, mut widget| {
            widget.update_style(|style| apply_fn(value, style));
            widget.request_layout();
        });
        ctx.update_style(|style| apply_fn(value, style));
    }
}
