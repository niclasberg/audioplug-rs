use crate::{
    core::WindowTheme,
    ui::{
        AppState, View, WidgetId,
        reactive::{CanCreate, Owner, ReadSignal},
    },
};

pub struct ScopeContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl ScopeContext<'_> {
    pub fn theme(&self) -> ReadSignal<WindowTheme> {
        self.app_state.theme_signal.as_read_signal()
    }
}

impl<'s> CanCreate<'s> for ScopeContext<'s> {
    fn create_context(&'s mut self) -> crate::ui::reactive::CreateContext<'s> {
        self.app_state.create_context(Owner::Widget(self.id))
    }
}

pub struct Stateful<F> {
    f: F,
}

impl<V, F> Stateful<F>
where
    V: View,
    F: FnOnce(ScopeContext) -> V,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<V, F> View for Stateful<F>
where
    V: View,
    F: FnOnce(ScopeContext) -> V + 'static,
{
    type Element = V::Element;

    fn build(self, ctx: &mut crate::ui::BuildContext<Self::Element>) -> Self::Element {
        let inner_view = (self.f)(ScopeContext {
            id: ctx.id().id,
            app_state: ctx.app_state,
        });
        inner_view.build(ctx)
    }
}
