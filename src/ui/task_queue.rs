use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use super::reactive::NodeId;
use crate::ui::{
    AppState, Widget, WidgetId, WidgetMut,
    reactive::{EffectContext, EffectFn, HandleEventFn, WatchContext, WatchFn},
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
        f: Weak<RefCell<WatchFn>>,
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
                    let mut cx = EffectContext {
                        effect_id: id,
                        app_state,
                    };
                    (RefCell::borrow_mut(&f))(&mut cx);
                    app_state.reactive_graph.mark_node_as_clean(id);
                }
            }
            Task::UpdateBinding { f, node_id } => {
                if let Some(f) = f.upgrade() {
                    (RefCell::borrow_mut(&f))(&mut WatchContext { app_state });
                    app_state.reactive_graph.mark_node_as_clean(node_id);
                }
            }
            Task::HandleEvent { f, event } => {
                if let Some(f) = f.upgrade() {
                    f(&mut WatchContext { app_state }, &event);
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
