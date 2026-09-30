use std::{
    any::Any,
    collections::VecDeque,
    ops::DerefMut,
    rc::{Rc, Weak},
};

use super::reactive::NodeId;
use crate::ui::{
    AppState, Widget, WidgetId, WidgetMut,
    reactive::{EffectContext, EffectState, HandleEventFn, runtime::NodeType},
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
            Task::RunEffect { id } => {
                if let Some(mut node) = app_state.reactive_graph.lease_node(id) {
                    let NodeType::Effect(effect_state) = node.deref_mut() else {
                        unreachable!()
                    };

                    match effect_state {
                        EffectState::EffectFn { f, dynamic_sources } => f(&mut EffectContext {
                            effect_id: id,
                            app_state,
                            track_reads: *dynamic_sources,
                        }),
                        EffectState::WidgetPropBinding {
                            widget_id,
                            prop,
                            read_scope,
                        } => {
                            // Widget might have been removed
                            if !app_state.widgets.contains(*widget_id) {
                                return;
                            }

                            // Effect node might have been removed
                            if !app_state.reactive_graph.contains(id) {
                                return;
                            }

                            app_state.widgets.apply_widget_prop(
                                &mut app_state.reactive_graph,
                                *widget_id,
                                prop,
                                *read_scope,
                            );
                        }
                    }

                    app_state.reactive_graph.unlease_node(node);
                    app_state.reactive_graph.mark_node_as_clean(id);
                }
            }
            Task::HandleEvent { f, event } => {
                /*if let Some(f) = f.upgrade() {
                    f(&mut WatchContext { app_state }, &event);
                }*/
                todo!()
            }
            Task::UpdateWidget { widget_id, f } => {
                if app_state.widgets.contains(widget_id) {
                    f(WidgetMut::new(app_state, widget_id))
                }
            }
        }
    }
}
