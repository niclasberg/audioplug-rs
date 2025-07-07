use crate::{
    app::{AppState, CreateContext, ReactiveContext, ReadSignal, View, ViewContext, WidgetId},
    core::WindowTheme,
};

pub struct ScopeContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl ScopeContext<'_> {
    pub fn theme(&self) -> ReadSignal<WindowTheme> {
        self.app_state.window_theme_signal(self.id)
    }
}

impl ReactiveContext for ScopeContext<'_> {
    fn runtime(&self) -> &crate::app::Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut crate::app::Runtime {
        self.app_state.runtime_mut()
    }
}

impl CreateContext for ScopeContext<'_> {
    fn owner(&self) -> Option<crate::app::Owner> {
        Some(crate::app::Owner::Widget(self.id))
    }
}

impl ViewContext for ScopeContext<'_> {
    fn window_id(&self) -> crate::app::WindowId {
        self.app_state.get_window_id_for_widget(self.id)
    }
}

pub struct Stateful<F> {
    f: F,
}

impl<V, F> Stateful<F>
where
    V: View,
    F: FnOnce(&mut ScopeContext) -> V,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<V, F> View for Stateful<F>
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
