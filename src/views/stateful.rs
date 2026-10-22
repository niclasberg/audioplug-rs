use crate::ui::{
    AppState, View, WidgetId, Widgets,
    prelude::CanRead,
    reactive::{CanCreate, CreateContext, Owner, ReactiveGraph, ReadContext, ReadScope},
};

pub struct ScopeContext<'a> {
    id: WidgetId,
    widgets: &'a mut Widgets,
    reactive_graph: &'a mut ReactiveGraph,
}

impl<'s> CanCreate<'s> for ScopeContext<'s> {
    fn create_context<'s2>(&'s2 mut self) -> CreateContext<'s2>
    where
        's: 's2,
    {
        CreateContext {
            widgets: &mut self.widgets,
            reactive_graph: &mut self.reactive_graph,
            owner: Owner::Widget(self.id),
        }
    }
}

impl<'s> CanRead<'s> for ScopeContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> crate::ui::reactive::ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Untracked,
            current_widget: Some(self.id),
        }
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
    fn build(self, cx: &mut crate::ui::BuildContext) {
        let inner_view = (self.f)(&mut ScopeContext {
            id: cx.id(),
            widgets: cx.widgets,
            reactive_graph: cx.reactive_graph,
        });
        inner_view.build(cx);
    }
}
