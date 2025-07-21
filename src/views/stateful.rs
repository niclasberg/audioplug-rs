use crate::{
    core::WindowTheme,
    ui::{AppState, CreateContext, ReactiveContext, ReadSignal, View, ViewContext, WidgetId},
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
    fn app_state(&self) -> &AppState {
        self.app_state
    }

    fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
    }
}

impl CreateContext for ScopeContext<'_> {
    fn owner(&self) -> Option<crate::ui::Owner> {
        Some(crate::ui::Owner::Widget(self.id))
    }
}

impl ViewContext for ScopeContext<'_> {
    fn window_id(&self) -> crate::ui::WindowId {
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

    fn build(self, ctx: &mut crate::ui::BuildContext<Self::Element>) -> Self::Element {
        let inner_view = (self.f)(&mut ScopeContext {
            id: ctx.id().id,
            app_state: ctx.app_state,
        });
        inner_view.build(ctx)
    }
}
