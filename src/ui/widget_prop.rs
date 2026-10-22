use bitflags::bitflags;
use parley::{FontStyle, FontWeight, FontWidth};
use slotmap::SecondaryMap;
use taffy::{AlignSelf, JustifySelf};

use crate::{
    core::{Color, Cursor, Size},
    ui::{
        Prop, WidgetId, Widgets,
        reactive::{ReactiveGraph, ReactiveValue, ReadContext, ReadScope},
        render::ShadowOptions,
        style::{Border, Edges, Fill, Length},
    },
};

pub type WidgetPropMap = SecondaryMap<WidgetId, WidgetProps>;

#[derive(Default)]
pub struct WidgetProps {
    props: Vec<WidgetProp>,
}

impl WidgetProps {
    pub fn insert(&mut self, prop: WidgetProp) {
        if let Some(existing) = self.props.iter_mut().find(|p| p.is_same_kind(&prop)) {
            *existing = prop;
        } else {
            self.props.push(prop);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &WidgetProp> {
        self.props.iter()
    }

    pub fn apply(
        &self,
        reactive_graph: &mut ReactiveGraph,
        widget_id: WidgetId,
        widgets: &mut Widgets,
    ) {
        let mut invalidation = Invalidation::empty();
        for prop in self.props.iter() {
            invalidation |= prop.apply(reactive_graph, widget_id, widgets);
        }

        if invalidation.contains(Invalidation::LAYOUT) {
            widgets.request_layout(widget_id);
        }
        if invalidation.contains(Invalidation::RENDER) {
            widgets.request_render(widget_id);
        }
    }

    pub fn remove_const_props(&mut self) {
        self.props.retain(|p| !p.is_const());
    }
}

bitflags! {
    pub struct Invalidation: u8 {
        const RENDER = 1;
        const LAYOUT = 1 << 1;
    }
}

macro_rules! define_widget_props {
    ($($name:ident => $t:ty), +) => {
        pub enum WidgetProp {
            $($name(Prop<$t>),)*
        }

        impl WidgetProp {
            pub fn is_same_kind(&self, other: &Self) -> bool {
                match (self, other) {
                    $((Self::$name(_), Self::$name(_)) => true,)*
                    _ => false
                }
            }

            pub fn is_const(&self) -> bool {
                match self {
                    $(Self::$name(Prop::Const(_)) => true,)*
                    _ => false
                }
            }
        }
    };
}

define_widget_props! {
    Hidden => bool,
    Padding => Edges,
    Width => Length,
    Height => Length,
    MinWidth => Length,
    MinHeight => Length,
    MaxWidth => Length,
    MaxHeight => Length,
    AspectRatio => f32,
    Border => Border,
    Margin => Edges,
    Inset => Edges,
    Background => Fill,
    CornerRadius => Size,
    JustifySelf => JustifySelf,
    AlignSelf => AlignSelf,
    BoxShadow => ShadowOptions,
    FlexGrow => f32,
    FlexShrink => f32,
    Cursor => Cursor,
    Text => String,
    TextColor => Color,
    FontSize => f32,
    FontWeight => FontWeight,
    FontStyle => FontStyle,
    FontWidth => FontWidth
}

impl WidgetProp {
    pub fn apply(
        &self,
        reactive_graph: &mut ReactiveGraph,
        widget_id: WidgetId,
        widgets: &mut Widgets,
    ) -> Invalidation {
        let mut cx = ReadContext {
            widgets,
            reactive_graph,
            scope: ReadScope::SyncWidget(widget_id),
            current_widget: Some(widget_id),
        };

        fn replace<T: PartialEq>(dst: &mut T, new_val: T) -> bool {
            if *dst != new_val {
                *dst = new_val;
                true
            } else {
                false
            }
        }

        match self {
            WidgetProp::Hidden(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).hidden, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::Padding(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).padding, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::Width(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).width, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::Height(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).height, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::MinWidth(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).min_width, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::MinHeight(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).min_height, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::MaxWidth(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).max_width, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::MaxHeight(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).max_height, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::AspectRatio(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).aspect_ratio, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::Margin(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).margin, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::Inset(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).margin, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::Background(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).background, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::CornerRadius(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).corner_radius, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::Border(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).border, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::JustifySelf(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).justify_self, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::AlignSelf(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).align_self, value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::BoxShadow(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).box_shadow, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::FlexGrow(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).flex_grow, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::FlexShrink(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.style_mut(widget_id).flex_shrink, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::Cursor(prop) => {
                let value = Some(prop.get(&mut cx));
                if replace(&mut widgets.style_mut(widget_id).cursor, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::Text(prop) => {
                let value = prop.get(&mut cx);
                widgets.set_text(widget_id, value)
            }
            WidgetProp::TextColor(prop) => {
                let value = prop.get(&mut cx);
                if replace(&mut widgets.text_mut(widget_id).style.color, value) {
                    return Invalidation::RENDER;
                }
            }
            WidgetProp::FontSize(prop) => {
                let value = prop.get(&mut cx);
                if widgets.text_mut(widget_id).set_font_size(value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::FontWeight(prop) => {
                let value = prop.get(&mut cx);
                if widgets.text_mut(widget_id).set_font_weight(value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::FontStyle(prop) => {
                let value = prop.get(&mut cx);
                if widgets.text_mut(widget_id).set_font_style(value) {
                    return Invalidation::LAYOUT;
                }
            }
            WidgetProp::FontWidth(prop) => {
                let value = prop.get(&mut cx);
                if widgets.text_mut(widget_id).set_font_width(value) {
                    return Invalidation::LAYOUT;
                }
            }
        };
        Invalidation::empty()
    }
}
