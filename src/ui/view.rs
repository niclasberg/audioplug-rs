use super::{
    AppState, CallbackContext, CreateContext, EventStatus, MouseEventContext, Owner, ParamContext,
    ReactiveContext, ReadContext, Scope, TypedWidgetId, ViewContext, ViewSequence, Widget,
    WidgetFlags, WidgetId, WrappedWidget,
    app_state::WidgetInsertPos,
    overlay::OverlayOptions,
    style::{Style, StyleBuilder},
};
use crate::{MouseButton, MouseEvent};
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

    fn on_click<F>(self, f: F) -> impl View
    where
        Self: Sized,
        F: Fn(&mut CallbackContext) + 'static,
    {
        OnClick {
            parent_view: self,
            on_click_fn: f,
        }
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

pub struct OnClick<V, F> {
    parent_view: V,
    on_click_fn: F,
}

impl<V: View, F: Fn(&mut CallbackContext) + 'static> View for OnClick<V, F> {
    type Element = OnClickWidget<V::Element, F>;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let parent = cx.build_inner(self.parent_view);
        OnClickWidget {
            parent,
            on_click_fn: self.on_click_fn,
        }
    }
}

pub struct OnClickWidget<W, F> {
    parent: W,
    on_click_fn: F,
}

impl<W: Widget, F: Fn(&mut CallbackContext) + 'static> WrappedWidget for OnClickWidget<W, F> {
    type Inner = W;

    fn inner(&self) -> &Self::Inner {
        &self.parent
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.parent
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down {
                button: MouseButton::LEFT,
                position,
                ..
            } if cx.bounds().contains(position) => {
                cx.capture_mouse();
                EventStatus::Handled
            }
            MouseEvent::Up {
                button: MouseButton::LEFT,
                position,
                ..
            } if cx.has_mouse_capture() => {
                cx.release_capture();
                if cx.bounds().contains(position) {
                    (self.on_click_fn)(&mut cx.as_callback_context());
                }
                EventStatus::Handled
            }
            _ => self.parent.mouse_event(event, cx),
        }
    }
}

pub struct BuildContext<'a, W: Widget + ?Sized> {
    id: WidgetId,
    pub(crate) app_state: &'a mut AppState,
    pub(super) style_builder: &'a mut StyleBuilder,
    _phantom: PhantomData<W>,
}

impl<'a, W: Widget + ?Sized> BuildContext<'a, W> {
    pub fn new(
        id: WidgetId,
        app_state: &'a mut AppState,
        style_builder: &'a mut StyleBuilder,
    ) -> Self {
        Self {
            id,
            app_state,
            style_builder,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> TypedWidgetId<W> {
        TypedWidgetId::new(self.id)
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.app_state
            .widget_data_mut(self.id)
            .set_or_clear_flag(WidgetFlags::FOCUSABLE, focusable);
    }

    pub fn add_child(&mut self, view: impl View) -> WidgetId {
        self.app_state
            .add_widget(self.id, view, WidgetInsertPos::End)
    }

    pub fn add_children(&mut self, view_sequence: impl ViewSequence) {
        view_sequence.build_seq(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
            style_builder: self.style_builder,
            _phantom: PhantomData,
        });
    }

    pub fn add_overlay(&mut self, view: impl View, options: OverlayOptions) -> WidgetId {
        self.app_state
            .add_widget(self.id, view, WidgetInsertPos::Overlay(options))
    }

    pub(crate) fn build_inner<V: View>(&mut self, view: V) -> V::Element {
        view.build(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
            style_builder: self.style_builder,
            _phantom: PhantomData,
        })
    }

    pub fn apply_style(&mut self, style_fn: impl FnOnce(&mut StyleBuilder)) {
        style_fn(self.style_builder);
    }

    pub fn set_default_style(&mut self, style: Style) {
        self.app_state.widget_data_mut(self.id).style = style;
    }

    pub fn update_default_style(&mut self, f: impl FnOnce(&mut Style)) {
        f(&mut self.app_state.widget_data_mut(self.id).style);
    }
}

impl<W: Widget + ?Sized> ParamContext for BuildContext<'_, W> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl<W: Widget + ?Sized> ReadContext for BuildContext<'_, W> {
    fn scope(&self) -> Scope {
        Scope::Root
    }
}

impl<W: Widget + ?Sized> ReactiveContext for BuildContext<'_, W> {
    fn runtime(&self) -> &super::Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut super::Runtime {
        self.app_state.runtime_mut()
    }
}

impl<W: Widget + ?Sized> CreateContext for BuildContext<'_, W> {
    fn owner(&self) -> Option<Owner> {
        Some(Owner::Widget(self.id))
    }
}

impl<W: Widget + ?Sized> ViewContext for BuildContext<'_, W> {
    fn window_id(&self) -> super::WindowId {
        self.app_state.get_window_id_for_widget(self.id)
    }
}
