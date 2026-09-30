use parley::FontContext;

use super::{
    AppState, ViewSequence, Widget, WidgetFlags, WidgetHandle, WidgetId, WidgetPos,
    overlay::OverlayOptions,
    reactive::{CanCreate, CanRead, Owner, ReadScope},
};
use crate::{
    core::TextLayoutContext,
    ui::{
        Prop, ViewStyle, ViewText, WidgetData,
        reactive::{CreateContext, ReadContext},
        style::{DisplayStyle, Style},
        text::{TextContext, TextData},
    },
};
use std::marker::PhantomData;

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: 'static {
    type Element: Widget + 'static;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element;

    fn into_any_view(self) -> AnyView
    where
        Self: Sized + 'static,
    {
        Box::new(move |ctx| Box::new(ctx.build_inner(self)))
    }
}

impl View for AnyView {
    type Element = Box<dyn Widget>;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        self(ctx)
    }

    fn into_any_view(self) -> AnyView {
        self
    }
}

pub struct BuildContext<'a, W: Widget + ?Sized> {
    id: WidgetId,
    pub(crate) app_state: &'a mut AppState,
    _phantom: PhantomData<W>,
}

impl<'a, W: Widget + ?Sized> BuildContext<'a, W> {
    pub fn new(id: WidgetId, app_state: &'a mut AppState) -> Self {
        Self {
            id,
            app_state,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> WidgetHandle<W> {
        WidgetHandle::new(self.id)
    }

    fn widget_data_mut(&mut self) -> &mut WidgetData {
        &mut self.app_state.widgets.tree[self.id]
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.widget_data_mut()
            .set_or_clear_flag(WidgetFlags::FOCUSABLE, focusable);
    }

    pub fn set_clickable(&mut self, clickable: bool) {
        self.widget_data_mut()
            .set_or_clear_flag(WidgetFlags::CLICKABLE, clickable);
    }

    pub fn set_draggable(&mut self, draggable: bool) {
        self.widget_data_mut()
            .set_or_clear_flag(WidgetFlags::DRAGGABLE, draggable);
    }

    pub fn set_display_style(&mut self, display_style: DisplayStyle) {
        self.widget_data_mut().display_style = display_style;
    }

    pub fn add_child(&mut self, view: impl View) -> WidgetId {
        self.app_state
            .add_widget(view, WidgetPos::LastChild(self.id))
    }

    pub fn add_children(&mut self, view_sequence: impl ViewSequence) {
        view_sequence.build_seq(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
            _phantom: PhantomData,
        });
    }

    pub fn add_overlay(&mut self, view: impl View, options: OverlayOptions) -> WidgetId {
        self.app_state
            .add_widget(view, WidgetPos::Overlay(self.id, options))
    }

    pub fn apply_style(&mut self, style: ViewStyle) {
        fn _inner(cx: &mut BuildContext<dyn Widget>, style: ViewStyle) {
            apply_layout_style(style.aspect_ratio, cx, |value, style| {
                style.aspect_ratio = Some(value);
            });
            apply_render_style(style.background, cx, |value, style| {
                style.background = Some(value);
            });
            apply_layout_style(style.border, cx, |value, style| {
                style.border = value;
            });
            apply_layout_style(style.corner_radius, cx, |value, style| {
                style.corner_radius = value;
            });
            apply_layout_style(style.height, cx, |value, style| {
                style.height = value;
            });
            apply_layout_style(style.hidden, cx, |value, style| {
                style.hidden = value;
            });
            apply_layout_style(style.min_height, cx, |value, style| {
                style.min_height = value;
            });
            apply_layout_style(style.min_width, cx, |value, style| {
                style.min_width = value;
            });
            apply_layout_style(style.max_height, cx, |value, style| {
                style.max_height = value;
            });
            apply_layout_style(style.max_width, cx, |value, style| {
                style.max_width = value;
            });
            apply_layout_style(style.padding, cx, |value, style| {
                style.padding = value;
            });
            apply_layout_style(style.width, cx, |value, style| {
                style.width = value;
            });
            apply_render_style(style.border_color, cx, |value, style| {
                style.border_color = Some(value);
            });
            apply_layout_style(style.align_self, cx, |value, style| {
                style.align_self = Some(value);
            });
            apply_layout_style(style.flex_grow, cx, |value, style| style.flex_grow = value);
            apply_layout_style(style.flex_shrink, cx, |value, style| {
                style.flex_shrink = value
            });
            apply_render_style(style.box_shadow, cx, |value, style| {
                style.box_shadow = Some(value);
            })
        }
        let mut cx = BuildContext {
            id: self.id,
            app_state: self.app_state,
            _phantom: PhantomData,
        };
        _inner(&mut cx, style);
    }

    pub fn apply_text(&mut self, text: ViewText) {
        let mut text_data = TextData::new();
        let value = if let Some(value) = text.text {
            value.get_and_bind(self, |value, mut widget| {
                widget.update_text(|t| t.set_text(value));
                widget.request_layout();
            })
        } else {
            "".to_string()
        };
        text_data.set_text(value);
        self.app_state.widgets.texts.insert(self.id, text_data);
    }

    pub(crate) fn build_inner<V: View>(&mut self, view: V) -> V::Element {
        view.build(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
            _phantom: PhantomData,
        })
    }

    pub fn text_context(&mut self) -> TextContext<'_> {
        TextContext::new(
            &mut self.app_state.font_cx,
            &mut self.app_state.text_layout_cx,
            &mut self.app_state.glyph_cache,
        )
    }
}

fn apply_layout_style<T: Clone + 'static>(
    accessor: Option<Prop<T>>,
    cx: &mut BuildContext<dyn Widget>,
    apply_fn: fn(T, &mut Style),
) {
    if let Some(accessor) = accessor {
        let value = accessor.get_and_bind(cx, move |value, mut widget| {
            widget.update_style(|style| apply_fn(value, style));
            widget.request_layout();
        });
        apply_fn(value, &mut cx.widget_data_mut().style)
    }
}

fn apply_render_style<T: Clone + 'static>(
    accessor: Option<Prop<T>>,
    cx: &mut BuildContext<dyn Widget>,
    apply_fn: fn(T, &mut Style),
) {
    if let Some(accessor) = accessor {
        let value = accessor.get_and_bind(cx, move |value, mut widget| {
            widget.update_style(|style| apply_fn(value, style));
            widget.request_render();
        });
        apply_fn(value, &mut cx.widget_data_mut().style)
    }
}

impl<'a, W: Widget + ?Sized> CanRead<'a> for BuildContext<'a, W> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        'a: 's2,
    {
        self.app_state
            .read_context(ReadScope::Untracked, Some(self.id))
    }
}

impl<'s, W: Widget + ?Sized> CanCreate<'s> for BuildContext<'s, W> {
    fn create_context<'s2>(&'s2 mut self) -> CreateContext<'s2>
    where
        's: 's2,
    {
        self.app_state.create_context(Owner::Widget(self.id))
    }
}
