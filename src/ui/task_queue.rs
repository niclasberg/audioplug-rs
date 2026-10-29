use std::collections::VecDeque;

use crate::ui::{AppState, WidgetId, WidgetMut};

#[derive(Default)]
pub struct TaskQueue {
    pub(super) tasks: VecDeque<Task>,
}

impl TaskQueue {
    pub(crate) fn push(&mut self, task: Task) {
        self.tasks.push_back(task);
    }
}

pub enum Task {
    UpdateWidget {
        widget_id: WidgetId,
        f: Box<dyn FnOnce(WidgetMut<'_>)>,
    },
}

impl Task {
    pub(super) fn run(self, app_state: &mut AppState) {
        match self {
            Task::UpdateWidget { widget_id, f } => {
                if app_state.ui.contains_widget(widget_id) {
                    f(WidgetMut::new(
                        &mut app_state.ui,
                        &mut app_state.widget_handlers,
                        widget_id,
                    ))
                }
            }
        }
    }
}
