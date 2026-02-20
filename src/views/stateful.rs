use crate::{
    core::WindowTheme,
    ui::{
        AppState, CreateContext, ReactiveContext, ReactiveGraph, ReadSignal, TaskQueue, View,
        WidgetId, Widgets, reactive::ReactiveContextMut,
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

impl ReactiveContext for ScopeContext<'_> {
    fn reactive_graph_and_widgets(&self) -> (&ReactiveGraph, &Widgets) {
        self.app_state.reactive_graph_and_widgets()
    }

    fn reactive_graph_mut_and_widgets(&mut self) -> (&mut ReactiveGraph, &Widgets) {
        self.app_state.reactive_graph_mut_and_widgets()
    }
}

impl ReactiveContextMut for ScopeContext<'_> {
    fn components_mut(&mut self) -> (&mut ReactiveGraph, &mut Widgets, &mut TaskQueue) {
        self.app_state.components_mut()
    }
}

impl CreateContext for ScopeContext<'_> {
    fn owner(&self) -> crate::ui::Owner {
        crate::ui::Owner::Widget(self.id)
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
