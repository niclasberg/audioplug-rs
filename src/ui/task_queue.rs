use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use super::reactive::{
    EffectContext, NodeId, ReactiveContext, ReactiveGraph, ReadContext, ReadScope, WatchContext,
    WidgetContext, WriteContext,
};
use crate::ui::{
    AnyView, AppState, Widget, WidgetId, WidgetMut, WidgetRef, Widgets,
    reactive::{BindingFn, EffectFn, HandleEventFn, ReactiveContextMut},
};

#[derive(Default)]
pub struct TaskQueue(pub(super) VecDeque<Task>);

impl TaskQueue {
    pub(crate) fn push(&mut self, task: Task) {
        self.0.push_back(task);
    }
}

pub enum Task {
    RunEffect {
        id: NodeId,
        f: Weak<RefCell<EffectFn>>,
    },
    UpdateBinding {
        f: Weak<RefCell<BindingFn>>,
        node_id: NodeId,
    },
    UpdateWidget {
        widget_id: WidgetId,
        f: Box<dyn FnOnce(WidgetMut<'_, dyn Widget>)>,
    },
    HandleEvent {
        f: Weak<HandleEventFn>,
        event: Rc<dyn Any>,
    },
}

impl Task {
    pub(super) fn run(self, app_state: &mut AppState) {
        match self {
            Task::RunEffect { id, f } => {
                if let Some(f) = f.upgrade() {
                    let mut cx = EffectContextImpl {
                        effect_id: id,
                        app_state,
                    };
                    (RefCell::borrow_mut(&f))(&mut cx);
                    app_state.reactive_graph.mark_node_as_clean(id);
                }
            }
            Task::UpdateBinding { f, node_id } => {
                if let Some(f) = f.upgrade() {
                    (RefCell::borrow_mut(&f))(app_state);
                    app_state.reactive_graph.mark_node_as_clean(node_id);
                }
            }
            Task::HandleEvent { f, event } => {
                if let Some(f) = f.upgrade() {
                    f(app_state, &event);
                }
            }
            Task::UpdateWidget { widget_id, f } => {
                if app_state.widgets.contains(widget_id) {
                    f(WidgetMut::new(app_state, widget_id))
                }
            }
        }
    }
}

pub struct EffectContextImpl<'a> {
    pub(super) effect_id: NodeId,
    pub(super) app_state: &'a mut AppState,
}

impl EffectContext for EffectContextImpl<'_> {
    fn as_watch_context(&mut self) -> &mut dyn WatchContext {
        self.app_state
    }
}

impl WidgetContext for EffectContextImpl<'_> {
    fn widget_ref_dyn(&self, id: WidgetId) -> WidgetRef<'_, dyn Widget> {
        WidgetRef::new(&self.app_state.widgets, id)
    }

    fn widget_mut_dyn(&mut self, id: WidgetId) -> WidgetMut<'_, dyn Widget> {
        WidgetMut::new(self.app_state, id)
    }

    fn replace_widget_dyn(&mut self, id: WidgetId, view: AnyView) {
        self.app_state.replace_widget(id, view);
    }
}

impl ReactiveContext for EffectContextImpl<'_> {
    fn reactive_graph_and_widgets(&self) -> (&ReactiveGraph, &Widgets) {
        self.app_state.reactive_graph_and_widgets()
    }

    fn reactive_graph_mut_and_widgets(&mut self) -> (&mut ReactiveGraph, &Widgets) {
        self.app_state.reactive_graph_mut_and_widgets()
    }
}

impl ReactiveContextMut for EffectContextImpl<'_> {
    fn components_mut(&mut self) -> (&mut ReactiveGraph, &mut Widgets, &mut TaskQueue) {
        self.app_state.components_mut()
    }
}

impl ReadContext for EffectContextImpl<'_> {
    fn scope(&self) -> ReadScope {
        ReadScope::Node(self.effect_id)
    }
}

impl WriteContext for EffectContextImpl<'_> {}
