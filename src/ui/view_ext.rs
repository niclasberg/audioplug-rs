use taffy::AlignSelf;

use crate::{
    core::{Color, Cursor, Size},
    ui::{
        Prop,
        render::ShadowOptions,
        style::{Border, Edges, Fill, Length},
        widget_prop::{WidgetProp, WidgetProps},
    },
};

pub trait ViewExt: Sized {
    fn props_mut(&mut self) -> &mut WidgetProps;
}

pub trait StyleExt: ViewExt {
    fn hidden(mut self, value: impl Into<Prop<bool>>) -> Self {
        self.props_mut().insert(WidgetProp::Hidden(value.into()));
        self
    }

    fn padding(mut self, value: impl Into<Prop<Edges>>) -> Self {
        self.props_mut().insert(WidgetProp::Padding(value.into()));
        self
    }

    fn margin(mut self, value: impl Into<Prop<Edges>>) -> Self {
        self.props_mut().insert(WidgetProp::Margin(value.into()));
        self
    }

    fn height(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.props_mut().insert(WidgetProp::Height(value.into()));
        self
    }

    fn width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.props_mut().insert(WidgetProp::Width(value.into()));
        self
    }

    fn min_width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.props_mut().insert(WidgetProp::MinWidth(value.into()));
        self
    }

    fn max_width(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.props_mut().insert(WidgetProp::MaxWidth(value.into()));
        self
    }

    fn min_height(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.props_mut().insert(WidgetProp::MinHeight(value.into()));
        self
    }

    fn max_height(mut self, value: impl Into<Prop<Length>>) -> Self {
        self.props_mut().insert(WidgetProp::MaxHeight(value.into()));
        self
    }

    fn background(mut self, value: impl Into<Prop<Fill>>) -> Self {
        self.props_mut()
            .insert(WidgetProp::Background(value.into()));
        self
    }

    fn corner_radius(mut self, value: impl Into<Prop<Size>>) -> Self {
        self.props_mut()
            .insert(WidgetProp::CornerRadius(value.into()));
        self
    }

    fn border(mut self, value: impl Into<Prop<Border>>) -> Self {
        self.props_mut().insert(WidgetProp::Border(value.into()));
        self
    }

    fn align_self(mut self, value: impl Into<Prop<AlignSelf>>) -> Self {
        self.props_mut().insert(WidgetProp::AlignSelf(value.into()));
        self
    }

    fn flex_grow(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.props_mut().insert(WidgetProp::FlexGrow(value.into()));
        self
    }

    fn flex_shrink(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.props_mut()
            .insert(WidgetProp::FlexShrink(value.into()));
        self
    }

    fn aspect_ratio(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.props_mut()
            .insert(WidgetProp::AspectRatio(value.into()));
        self
    }

    fn shadow(mut self, value: impl Into<Prop<ShadowOptions>>) -> Self {
        self.props_mut().insert(WidgetProp::BoxShadow(value.into()));
        self
    }

    fn cursor(mut self, value: impl Into<Prop<Cursor>>) -> Self {
        self.props_mut().insert(WidgetProp::Cursor(value.into()));
        self
    }
}

pub trait TextExt: ViewExt {
    fn text(mut self, value: impl Into<Prop<String>>) -> Self {
        self.props_mut().insert(WidgetProp::Text(value.into()));
        self
    }

    fn font_size(mut self, value: impl Into<Prop<f32>>) -> Self {
        self.props_mut().insert(WidgetProp::FontSize(value.into()));
        self
    }

    fn color(mut self, value: impl Into<Prop<Color>>) -> Self {
        self.props_mut().insert(WidgetProp::TextColor(value.into()));
        self
    }
}

impl ViewExt for WidgetProps {
    fn props_mut(&mut self) -> &mut WidgetProps {
        self
    }
}
impl StyleExt for WidgetProps {}
impl TextExt for WidgetProps {}
