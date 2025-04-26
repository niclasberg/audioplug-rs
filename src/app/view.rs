use super::{
    AppState, CreateContext, Owner, ParamContext, ReactiveContext, ReadContext, Scope,
    TypedWidgetId, ViewContext, Widget, WidgetFlags, WidgetId,
};
use crate::style::{Style, StyleBuilder};
use std::marker::PhantomData;

pub type AnyView = Box<dyn FnOnce(&mut BuildContext<Box<dyn Widget>>) -> Box<dyn Widget>>;

pub trait View: Sized + 'static {
    type Element: Widget + 'static;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element;

    fn into_any_view(self) -> AnyView
    where
        Self: 'static,
    {
        Box::new(move |ctx| Box::new(ctx.build(self)))
    }
}

impl View for AnyView {
    type Element = Box<dyn Widget>;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        self(ctx)
    }
}

pub struct BuildContext<'a, W: Widget> {
    id: WidgetId,
    pub(crate) app_state: &'a mut AppState,
    pub(super) style_builder: StyleBuilder,
    _phantom: PhantomData<W>,
}

impl<'a, W: Widget> BuildContext<'a, W> {
    pub fn new(id: WidgetId, app_state: &'a mut AppState) -> Self {
        Self {
            id,
            app_state,
            style_builder: Default::default(),
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
        self.app_state.add_widget(self.id, view)
    }

    pub fn add_overlay(&mut self, view: impl View) -> WidgetId {
        let child_id = self.add_child(view);
        self.app_state
            .widget_data_mut(child_id)
            .set_flag(WidgetFlags::OVERLAY);
        child_id
    }

    pub(crate) fn build<V: View>(&mut self, view: V) -> V::Element {
        let (widget, style_builder) = {
            let mut ctx = BuildContext {
                id: self.id,
                app_state: self.app_state,
                style_builder: Default::default(),
                _phantom: PhantomData,
            };
            (view.build(&mut ctx), ctx.style_builder)
        };
        self.style_builder.merge(style_builder);
        widget
    }

    pub fn apply_style(&mut self, style: StyleBuilder) {
        self.style_builder.merge(style);
    }

    pub fn set_default_style(&mut self, style: Style) {
        self.app_state.widget_data_mut(self.id).style = style;
    }

    pub fn update_default_style(&mut self, f: impl FnOnce(&mut Style)) {
        f(&mut self.app_state.widget_data_mut(self.id).style);
    }
}

impl<'s, W: Widget> ParamContext for BuildContext<'s, W> {
    fn host_handle(&self) -> &dyn super::HostHandle {
        self.app_state.host_handle()
    }
}

impl<'s, W: Widget> ReadContext for BuildContext<'s, W> {
    fn scope(&self) -> Scope {
        Scope::Root
    }
}

impl<'b, W: Widget> ReactiveContext for BuildContext<'b, W> {
    fn runtime(&self) -> &super::Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut super::Runtime {
        self.app_state.runtime_mut()
    }
}

impl<'s, W: Widget> CreateContext for BuildContext<'s, W> {
    fn owner(&self) -> Option<Owner> {
        Some(Owner::Widget(self.id))
    }
}

impl<'s, W: Widget> ViewContext for BuildContext<'s, W> {
    fn window_id(&self) -> super::WindowId {
        self.app_state.get_window_id_for_widget(self.id)
    }
}
