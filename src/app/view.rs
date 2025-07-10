use super::{
    AppState, CreateContext, Owner, ParamContext, ReactiveContext, ReadContext, Scope,
    TypedWidgetId, ViewContext, Widget, WidgetFlags, WidgetId,
};
use crate::{
    app::ViewSequence,
    style::{Style, StyleBuilder},
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
        Box::new(move |ctx| Box::new(ctx.build(self)))
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
        self.app_state.add_widget(self.id, view, None)
    }

    pub fn add_children(&mut self, view_sequence: impl ViewSequence) {
        view_sequence.build_seq(&mut BuildContext {
            id: self.id,
            app_state: self.app_state,
            style_builder: self.style_builder,
            _phantom: PhantomData,
        });
    }

    pub fn add_overlay(&mut self, view: impl View, z_index: usize) -> WidgetId {
        let child_id = self.add_child(view);
        self.app_state.make_widget_into_overlay(child_id, z_index);
        child_id
    }

    pub(crate) fn build<V: View>(&mut self, view: V) -> V::Element {
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
