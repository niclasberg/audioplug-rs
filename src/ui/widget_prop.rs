use parley::{FontStyle, FontWeight, FontWidth};
use slotmap::SecondaryMap;
use smallvec::SmallVec;
use taffy::{AlignSelf, JustifySelf};

use crate::{
    core::{Color, Cursor, Size},
    ui::{
        Prop, UiState, WidgetId,
        reactive::{CreateContext, Owner, Reactive, ReadContext, ReadScope, Readable, WidgetPhase},
        render::ShadowOptions,
        style::{Border, Edges, Fill, Length},
        ui_state::Invalidation,
    },
};

pub type WidgetBindingsMap = SecondaryMap<WidgetId, WidgetBindings>;

#[derive(Default)]
pub struct WidgetProps {
    props: Vec<WidgetProp>,
    pub(super) hidden: Option<Prop<bool>>,
    pub(super) enabled: Option<Prop<bool>>,
    pub(super) text_editable: Option<Prop<bool>>,
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

    pub fn install(self, widget_id: WidgetId, ui: &mut UiState) -> Option<WidgetBindings> {
        let mut bindings = SmallVec::new();
        let mut invalidation = Invalidation::empty();
        for prop in self.props {
            if let Some(binding) = prop.install(ui, widget_id, &mut invalidation) {
                bindings.push(binding);
            }
        }

        watch(widget_id, self.hidden, ui, move |ui, value| {
            ui.set_hidden(widget_id, value)
        });
        watch(widget_id, self.enabled, ui, move |ui, value| {
            ui.set_enabled(widget_id, value)
        });
        watch(widget_id, self.text_editable, ui, move |ui, value| {
            ui.set_text_editable(widget_id, value)
        });

        ui.invalidate_widget(widget_id, invalidation);

        if bindings.is_empty() {
            None
        } else {
            Some(WidgetBindings { bindings })
        }
    }
}

fn watch<T: Copy + 'static>(
    widget_id: WidgetId,
    prop: Option<Prop<T>>,
    ui: &mut UiState,
    f: impl Fn(&mut UiState, T) + 'static,
) {
    if let Some(prop) = prop {
        match prop.into_reactive() {
            Ok(reactive) => {
                reactive.watch(
                    &mut CreateContext::new(ui, Owner::Widget(widget_id)),
                    move |cx, value| f(&mut cx.ui, *value),
                );
            }
            Err(value) => f(ui, value),
        }
    }
}

pub struct WidgetBindings {
    bindings: SmallVec<[WidgetBinding; 2]>,
}

impl WidgetBindings {
    pub fn apply(&self, widget_id: WidgetId, ui: &mut UiState) {
        let node_id = ui.get_invalidation_node_id(widget_id, WidgetPhase::SyncWidget);
        let scope = ReadScope::Node(node_id);
        ui.clear_node_sources(node_id);
        let mut cx = ReadContext {
            ui,
            scope: scope,
            current_widget: Some(widget_id),
        };

        let mut invalidation = Invalidation::empty();
        for binding in self.bindings.iter() {
            invalidation |= binding.apply(&mut cx, widget_id);
        }
        ui.invalidate_widget(widget_id, invalidation);
        ui.mark_node_as_clean(node_id);
    }
}

macro_rules! define_prop {
    ($($name:ident: $t:ty => $apply:expr), +) => {
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

            pub fn install(self, ui: &mut UiState, id: WidgetId, inv: &mut Invalidation) -> Option<WidgetBinding> {
                match self {
                    $(Self::$name(p) => match p.into_reactive() {
                        Err(value) => { *inv = $apply(ui, id, value); None },
                        Ok(reactive) => Some(WidgetBinding::$name(reactive)),
                    },)*
                }
            }
        }

        pub enum WidgetBinding {
            $($name(Reactive<$t>),)*
        }

        impl WidgetBinding {
            pub fn apply(&self, cx: &mut ReadContext, id: WidgetId) -> Invalidation {
                match self {
                    $(Self::$name(reactive) => {
                        let value = reactive.get(cx);
                        $apply(cx.ui, id, value)
                    },)*
                }
            }
        }
    };
}

define_prop! {
    Padding: Edges => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).padding, value, Invalidation::LAYOUT),
    Width: Length => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).width, value, Invalidation::LAYOUT),
    Height: Length => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).height, value, Invalidation::LAYOUT),
    MinWidth: Length => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).min_width, value, Invalidation::LAYOUT),
    MinHeight: Length => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).min_height, value, Invalidation::LAYOUT),
    MaxWidth: Length => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).max_width, value, Invalidation::LAYOUT),
    MaxHeight: Length => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).max_height, value, Invalidation::LAYOUT),
    AspectRatio: f32 => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).aspect_ratio, Some(value), Invalidation::LAYOUT),
    Border: Border => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).border, Some(value), Invalidation::LAYOUT),
    Margin: Edges => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).margin, value, Invalidation::LAYOUT),
    Inset: Edges => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).inset, value, Invalidation::LAYOUT),
    Background: Fill => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).background, Some(value), Invalidation::LAYOUT | Invalidation::RENDER),
    CornerRadius: Size  => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).corner_radius, value, Invalidation::LAYOUT),
    JustifySelf: JustifySelf => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).justify_self, Some(value), Invalidation::LAYOUT),
    AlignSelf: AlignSelf => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).align_self, Some(value), Invalidation::LAYOUT),
    BoxShadow: ShadowOptions => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).box_shadow, Some(value), Invalidation::RENDER),
    FlexGrow: f32 => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).flex_grow, value, Invalidation::LAYOUT),
    FlexShrink: f32 => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).flex_shrink, value, Invalidation::LAYOUT),
    Cursor: Cursor => |ui: &mut UiState, id, value|
        replace(&mut ui.style_mut(id).cursor, Some(value), Invalidation::empty()),
    TextColor: Color => |ui: &mut UiState, id, value|
        replace(&mut ui.text_mut(id).color, value, Invalidation::TEXT | Invalidation::RENDER),
    FontSize: f32 => |ui: &mut UiState, id, value|
        replace(&mut ui.text_mut(id).font_size, value, Invalidation::LAYOUT | Invalidation::TEXT),
    FontWeight: FontWeight => |ui: &mut UiState, id, value|
        replace(&mut ui.text_mut(id).font_weight, Some(value), Invalidation::LAYOUT | Invalidation::TEXT),
    FontStyle: FontStyle => |ui: &mut UiState, id, value|
        replace(&mut ui.text_mut(id).font_style, Some(value), Invalidation::LAYOUT | Invalidation::TEXT),
    FontWidth: FontWidth => |ui: &mut UiState, id, value|
        replace(&mut ui.text_mut(id).font_width, Some(value), Invalidation::LAYOUT | Invalidation::TEXT),
    Text: String => |ui: &mut UiState, id, value|
        replace(&mut ui.text_mut(id).text, value, Invalidation::LAYOUT | Invalidation::TEXT)
}

fn replace<T: PartialEq>(dst: &mut T, new_val: T, invalidation: Invalidation) -> Invalidation {
    if *dst != new_val {
        *dst = new_val;
        invalidation
    } else {
        Invalidation::empty()
    }
}
