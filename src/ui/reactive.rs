mod accessor;
mod animation;
mod cached;
mod computed;
mod effect;
mod event_channel;
mod read_signal;
mod readable;
mod runtime;
mod signal_vec;
mod spring;
mod trigger;
mod tween;
mod var;
mod widget_status;

use std::{collections::VecDeque, rc::Rc};

pub use accessor::Accessor;
pub use animation::{Animated, AnimatedFn, Animation, Easing, SpringOptions, TweenOptions};
pub use cached::{Cached, CachedContext};
pub use computed::Computed;
pub(super) use effect::{BindingFn, EffectFn, EffectState};
pub use effect::{Effect, EffectContext, WatchContext};
pub(super) use event_channel::HandleEventFn;
pub use event_channel::{EventChannel, EventReceiver, create_event_channel};
pub use read_signal::ReadSignal;
pub use readable::*;
pub use runtime::*;
pub use trigger::Trigger;
pub use var::Var;
pub use widget_status::{FOCUS_STATUS, WidgetStatus, WidgetStatusFlags};

use crate::{
    core::FxHashSet,
    param::ParameterId,
    ui::{AppState, WidgetId, app_state::Task, reactive::effect::BindingState},
};

slotmap::new_key_type! {
    pub struct NodeId;
}

pub fn notify(app_state: &mut AppState, node_id: NodeId) {
    let mut observers = std::mem::take(&mut app_state.node_id_buffer);
    observers.clear();
    observers.extend(app_state.runtime.observers[node_id].iter());
    notify_source_changed(app_state, observers);
}

fn notify_source_changed(app_state: &mut AppState, mut nodes_to_notify: VecDeque<NodeId>) {
    let mut nodes_to_check = FxHashSet::default();

    {
        let direct_child_count = nodes_to_notify.len();
        let mut i = 0;
        while let Some(node_id) = nodes_to_notify.pop_front() {
            // Mark direct nodes as Dirty and grand-children as Check
            let new_state = if i < direct_child_count {
                NodeState::Dirty
            } else {
                NodeState::Check
            };
            let node = app_state.runtime.get_node_mut(node_id);
            if node.state < new_state {
                node.state = new_state;
                match &node.node_type {
                    NodeType::Effect(_) | NodeType::Binding(_) | NodeType::DerivedAnimation(_) => {
                        nodes_to_check.insert(node_id);
                    }
                    _ => {}
                }
                nodes_to_notify.extend(app_state.runtime.observers[node_id].iter());
            }
            i += 1;
        }
    }

    // Swap back the scratch buffer. Saves us from having to reallocate
    std::mem::swap(&mut app_state.node_id_buffer, &mut nodes_to_notify);

    for node_id in nodes_to_check {
        update_if_necessary(app_state, node_id);
    }
}

pub fn update_if_necessary(app_state: &mut AppState, node_id: NodeId) {
    if app_state.runtime.get_node(node_id).state == NodeState::Clean {
        return;
    }

    if app_state.runtime.get_node(node_id).state == NodeState::Check {
        for source_id in app_state.runtime.sources[node_id].clone() {
            if let SourceId::Node(source_id) = source_id {
                update_if_necessary(app_state, source_id);
                if app_state.runtime.get_node(node_id).state == NodeState::Dirty {
                    break;
                }
            }
        }
    }

    if app_state.runtime.get_node(node_id).state == NodeState::Dirty {
        let mut node_type = std::mem::replace(
            &mut app_state.runtime.get_node_mut(node_id).node_type,
            NodeType::TmpRemoved,
        );
        match &mut node_type {
            NodeType::Effect(EffectState { f }) => {
                // Clear the sources, they will be re-populated while running the effect function
                app_state.runtime.clear_node_sources(node_id);
                let task = Task::RunEffect {
                    id: node_id,
                    f: Rc::downgrade(f),
                };
                app_state.push_task(task);
            }
            NodeType::Binding(BindingState { f }) => {
                let task = Task::UpdateBinding {
                    f: Rc::downgrade(f),
                    node_id,
                };
                app_state.push_task(task);
            }
            NodeType::DerivedAnimation(anim) => {
                // Clear the sources, they will be re-populated while running the reset function
                app_state.runtime.clear_node_sources(node_id);
                if anim.reset(node_id, app_state) {
                    app_state.request_animation(anim.window_id, node_id);
                }
            }
            NodeType::Memo(memo) => {
                // Clear the sources, they will be re-populated while running the memo function
                app_state.runtime.clear_node_sources(node_id);
                if memo.eval(node_id, app_state) {
                    // Memo eval returned false, meaning that it has changed.
                    for &observer_id in app_state.runtime.observers[node_id].iter() {
                        app_state.runtime.nodes[observer_id].state = NodeState::Dirty;
                    }
                }
            }
            NodeType::Animation(..) => {
                panic!("Animations cannot depend on other reactive nodes")
            }
            NodeType::Trigger => panic!("Triggers cannot depend on other reactive nodes"),
            NodeType::Signal(_) => panic!("Signals cannot depend on other reactive nodes"),
            NodeType::EventEmitter => {
                panic!("Event emitters cannot depend on other reactive nodes")
            }
            NodeType::EventHandler(..) => {
                panic!("Event handlers should not be notified, use publish_event instead")
            }
            NodeType::TmpRemoved => panic!("Circular dependency?"),
        }
        std::mem::swap(
            &mut app_state.runtime.get_node_mut(node_id).node_type,
            &mut node_type,
        );
    }

    app_state.runtime.get_node_mut(node_id).state = NodeState::Clean;
}

pub(crate) fn notify_parameter_subscribers(app_state: &mut AppState, source_id: ParameterId) {
    let mut nodes_to_notify = std::mem::take(&mut app_state.node_id_buffer);
    nodes_to_notify.clear();
    nodes_to_notify.extend(
        app_state
            .runtime
            .parameter_observers
            .get_mut(&source_id)
            .unwrap()
            .iter(),
    );
    notify_source_changed(app_state, nodes_to_notify);
}

pub(crate) fn notify_widget_status_changed(
    app_state: &mut AppState,
    widget_id: WidgetId,
    change_mask: WidgetStatusFlags,
) {
    let mut nodes_to_notify = std::mem::take(&mut app_state.node_id_buffer);
    nodes_to_notify.clear();
    if let Some(widget_observers) = app_state.runtime.widget_observers.get(widget_id) {
        nodes_to_notify.extend(
            widget_observers
                .iter()
                .filter_map(|(node_id, mask)| mask.contains(change_mask).then_some(node_id)),
        );
    }
    notify_source_changed(app_state, nodes_to_notify);
}
