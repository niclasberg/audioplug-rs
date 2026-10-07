use taffy::{AlignSelf, JustifySelf};

use crate::{
    core::{Color, Cursor, Size},
    ui::{
        Paint, Prop, WidgetId,
        reactive::CreateContext,
        render::ShadowOptions,
        style::{Length, Edges},
    },
};

#[derive(Clone)]
pub enum WidgetProp {
    Hidden(Prop<bool>),
    Padding(Prop<Edges>),
    Width(Prop<Length>),
    Height(Prop<Length>),
    MinWidth(Prop<Length>),
    MinHeight(Prop<Length>),
    MaxWidth(Prop<Length>),
    MaxHeight(Prop<Length>),
    AspectRatio(Prop<f64>),
    Border(Prop<Length>),
    Margin(Prop<Edges>),
    Inset(Prop<Edges>),
    Background(Prop<Paint>),
    CornerRadius(Prop<Size>),
    BorderColor(Prop<Color>),
    JustifySelf(Prop<JustifySelf>),
    AlignSelf(Prop<AlignSelf>),
    BoxShadow(Prop<ShadowOptions>),
    FlexGrow(Prop<f32>),
    FlexShrink(Prop<f32>),
    Cursor(Prop<Cursor>),
}

impl WidgetProp {
    pub fn bind(&self, cx: &mut CreateContext, widget_id: WidgetId) {
        match self {
            WidgetProp::Hidden(prop) => todo!(),
            WidgetProp::Padding(prop) => todo!(),
            WidgetProp::Width(prop) => todo!(),
            WidgetProp::Height(prop) => todo!(),
            WidgetProp::MinWidth(prop) => todo!(),
            WidgetProp::MinHeight(prop) => todo!(),
            WidgetProp::MaxWidth(prop) => todo!(),
            WidgetProp::MaxHeight(prop) => todo!(),
            WidgetProp::AspectRatio(prop) => todo!(),
            WidgetProp::Border(prop) => todo!(),
            WidgetProp::Margin(prop) => todo!(),
            WidgetProp::Inset(prop) => todo!(),
            WidgetProp::Background(prop) => todo!(),
            WidgetProp::CornerRadius(prop) => todo!(),
            WidgetProp::BorderColor(prop) => todo!(),
            WidgetProp::JustifySelf(prop) => todo!(),
            WidgetProp::AlignSelf(prop) => todo!(),
            WidgetProp::BoxShadow(prop) => todo!(),
            WidgetProp::FlexGrow(prop) => todo!(),
            WidgetProp::FlexShrink(prop) => todo!(),
            WidgetProp::Cursor(prop) => todo!(),
        }
    }
}
