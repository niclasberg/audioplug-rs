use crate::{
    app::{AppState, CreateContext, ReactiveContext, ReadSignal, View, ViewContext, WidgetId},
    core::WindowTheme,
};

pub struct ScopeContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl<'a> ScopeContext<'a> {
    pub fn theme(&self) -> ReadSignal<WindowTheme> {
        self.app_state.window_theme_signal(self.id)
    }
}

impl<'a> ReactiveContext for ScopeContext<'a> {
    fn runtime(&self) -> &crate::app::Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut crate::app::Runtime {
        self.app_state.runtime_mut()
    }
}

impl<'a> CreateContext for ScopeContext<'a> {
    fn owner(&self) -> Option<crate::app::Owner> {
        Some(crate::app::Owner::Widget(self.id))
    }
}

impl<'a> ViewContext for ScopeContext<'a> {
    fn window_id(&self) -> crate::app::WindowId {
        self.app_state.get_window_id_for_widget(self.id)
    }
}

pub struct Scoped<F> {
    f: F,
}

impl<V, F> Scoped<F>
where
    V: View,
    F: FnOnce(&mut ScopeContext) -> V,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<V, F> View for Scoped<F>
where
    V: View,
    F: FnOnce(&mut ScopeContext) -> V + 'static,
{
    type Element = V::Element;

    fn build(self, ctx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        let inner_view = (self.f)(&mut ScopeContext {
            id: ctx.id().id,
            app_state: ctx.app_state,
        });
        inner_view.build(ctx)
    }
}
