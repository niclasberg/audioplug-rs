use crate::{
    core::WindowTheme,
    ui::{
        AppState, View, WidgetId,
        prelude::CanRead,
        reactive::{CanCreate, Owner, ReadScope, ReadSignal},
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
    fn create_context<'s2>(&'s2 mut self) -> crate::ui::reactive::CreateContext<'s2>
    where
        's: 's2,
    {
        self.app_state.create_context(Owner::Widget(self.id))
    }
}

impl<'s> CanRead<'s> for ScopeContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> crate::ui::reactive::ReadContext<'s2>
    where
        's: 's2,
    {
        self.app_state.read_context(ReadScope::Untracked)
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
