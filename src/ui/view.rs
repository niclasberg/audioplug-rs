use super::{
    AppState, ViewSequence, WidgetId, WidgetPos,
    overlay::OverlayOptions,
    reactive::{CanCreate, CanRead, Owner, ReadScope},
};
use crate::{
    core::Size,
    ui::{
        Capabilities, Prop, ViewEventHandlers, ViewStyle, ViewText, WidgetData,
        event_handler::WidgetHandler,
        reactive::{CreateContext, EffectContext, ReactiveValue, ReadContext},
        style::{DisplayStyle, Style},
        text::{TextContext, TextData},
    },
};

pub type AnyView = Box<dyn FnOnce(&mut BuildContext)>;

pub trait View: 'static {
    fn build(self, cx: &mut BuildContext);

    fn into_any_view(self) -> AnyView
    where
        Self: Sized + 'static,
    {
        Box::new(move |ctx| ctx.build_inner(self))
    }
}

impl View for AnyView {
    fn build(self, ctx: &mut BuildContext) {
        self(ctx)
    }

    fn into_any_view(self) -> AnyView {
        self
    }
}

pub struct BuildContext<'a> {
    id: WidgetId,
    pub(crate) app_state: &'a mut AppState,
}

impl<'a> BuildContext<'a> {
    pub fn new(id: WidgetId, app_state: &'a mut AppState) -> Self {
        Self { id, app_state }
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    fn widget_data_mut(&mut self) -> &mut WidgetData {
        &mut self.app_state.widgets.tree[self.id]
    }

    pub fn bind<T: 'static>(
        &mut self,
        prop: Prop<T>,
        mut f: impl FnMut(&mut EffectContext, WidgetId, &T) + 'static,
    ) -> &mut Self {
        let id = self.id;
        prop.watch(self, move |cx, value| {
            f(cx, id, value);
        });
        self
    }

    pub fn set_display_style(&mut self, display_style: DisplayStyle) {
        self.widget_data_mut().display_style = display_style;
    }

    pub fn intrinsic_size(&mut self, size: Size) -> &mut Self {
        self.widget_data_mut().intrinsic_size = Some(size);
        self
    }

    pub fn add_child(&mut self, view: impl View) -> WidgetId {
        self.app_state
            .add_widget(view, WidgetPos::LastChild(self.id))
    }

    pub fn add_children(&mut self, view_sequence: impl ViewSequence) {
        view_sequence.build_seq(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
        });
    }

    pub fn add_overlay(&mut self, view: impl View, options: OverlayOptions) -> WidgetId {
        self.app_state
            .add_widget(view, WidgetPos::Overlay(self.id, options))
    }

    pub fn style(&mut self, style: ViewStyle) -> &mut Self {
        fn _inner(cx: &mut BuildContext, style: ViewStyle) {
            apply_layout_style(style.aspect_ratio, cx, |value, style| {
                style.aspect_ratio = Some(value);
            });
            apply_render_style(style.background, cx, |value, style| {
                style.background = Some(value);
            });
            apply_layout_style(style.border, cx, |value, style| {
                style.border = Some(value);
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
            apply_layout_style(style.align_self, cx, |value, style| {
                style.align_self = Some(value);
            });
            apply_layout_style(style.flex_grow, cx, |value, style| style.flex_grow = value);
            apply_layout_style(style.flex_shrink, cx, |value, style| {
                style.flex_shrink = value
            });
            apply_render_style(style.box_shadow, cx, |value, style| {
                style.box_shadow = Some(value);
            });
            apply_render_style(style.cursor, cx, |value, style| {
                style.cursor = Some(value);
            });
        }
        let mut cx = BuildContext {
            id: self.id,
            app_state: self.app_state,
        };
        _inner(&mut cx, style);
        self
    }

    pub fn event_handlers(&mut self, event_handlers: ViewEventHandlers) -> &mut Self {
        for handler in event_handlers.handlers {
            self.app_state
                .event_handlers
                .push_event_handler(self.id, handler);
        }

        self.app_state
            .widgets
            .set_capabilities(self.id, event_handlers.capabilities);

        self
    }

    pub fn handler(
        &mut self,
        capabilities: Capabilities,
        handler: impl WidgetHandler,
    ) -> &mut Self {
        self.app_state
            .widgets
            .set_capabilities(self.id, capabilities);
        self.app_state
            .event_handlers
            .set_widget_handler(self.id, Box::new(handler));
        self
    }

    pub fn text(&mut self, text: ViewText) -> &mut Self {
        let mut text_data = TextData::new();
        let value = if let Some(value) = text.text {
            value.get_and_bind(self, |value, mut widget| {
                widget.update_text(|t| t.set_text(value));
                widget.request_layout();
            })
        } else {
            "".to_string()
        };
        if let Some(font_size) = text.font_size {
            let value = font_size.get_and_bind(self, |value, mut widget| {
                widget.update_text(|t| t.set_font_size(value));
                widget.request_layout();
            });
            text_data.set_font_size(value);
        };
        text_data.set_text(value);
        self.app_state.widgets.texts.insert(self.id, text_data);
        self
    }

    pub(crate) fn build_inner(&mut self, view: impl View) {
        view.build(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
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
    cx: &mut BuildContext,
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
    cx: &mut BuildContext,
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

impl<'a> CanRead<'a> for BuildContext<'a> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        'a: 's2,
    {
        self.app_state
            .read_context(ReadScope::Untracked, Some(self.id))
    }
}

impl<'s> CanCreate<'s> for BuildContext<'s> {
    fn create_context<'s2>(&'s2 mut self) -> CreateContext<'s2>
    where
        's: 's2,
    {
        self.app_state.create_context(Owner::Widget(self.id))
    }
}
