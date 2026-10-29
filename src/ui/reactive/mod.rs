mod animation;
mod cached;
mod computed;
mod contexts;
mod effect;
mod event_channel;
mod param;
mod reactive_value;
mod read_signal;
mod signal_vec;
mod spring;
mod trigger;
mod tween;
mod var;
mod view_status;

use std::any::Any;

pub use animation::{Animated, AnimatedVar, Animation, Easing, SpringOptions, TweenOptions};
pub use cached::{Cached, CachedContext};
pub use computed::Computed;
pub use contexts::*;
pub use effect::{Effect, EffectContext};
pub(super) use effect::{EffectFn, EffectState};
pub use event_channel::{EventChannel, EventReceiver, create_event_channel};
pub use param::ParamSetter;
pub use reactive_value::ReactiveValue;
pub use read_signal::ReadSignal;
pub use trigger::Trigger;
pub use var::Var;
pub use view_status::{WidgetStatus, WidgetStatusFlags};

use crate::{
    param::ParameterId,
    ui::{
        WidgetId,
        reactive::{
            animation::{AnimationState, DerivedAnimationState},
            cached::CachedState,
            var::SignalState,
        },
    },
};

slotmap::new_key_type! {
    pub struct NodeId;
}

pub struct Node {
    pub(crate) node_type: NodeType,
    pub(crate) state: NodeState,
    pub(crate) owner: Owner,
    /// Id of the first node that is owned by this node
    pub(crate) first_child_id: NodeId,
    /// Next node that has the same owner as this node
    pub(crate) next_sibling_id: NodeId,
}

/// Encodes the lifetime of the reactive node.
#[derive(Debug, Clone, Copy)]
pub enum Owner {
    /// The node is a root node, and will only be removed when the reactive graph is destroyed.
    Root,
    /// The reactive node is owned by a widget, and will be removed
    /// when the widget is removed.
    Widget(WidgetId),
    /// The reactive node is owned by another node (such as a Var being created within a memo)
    /// The node is removed when its parent is removed.
    Node(NodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Source {
    Parameter(ParameterId),
    Node(NodeId),
    Widget(WidgetId, WidgetStatusFlags),
}

#[derive(Debug, Clone, Copy)]
pub enum ReadScope {
    Untracked,
    Node(NodeId),
    Render(WidgetId),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub(crate) enum NodeState {
    /// Reactive value is valid, no need to recompute
    Clean = 0,
    /// Reactive value might be stale, check parent nodes to decide whether to recompute
    Check = 1,
    /// Reactive value is invalid, parents have changed, value needs to be recomputed
    Dirty = 2,
}

pub(crate) enum NodeType {
    // Temporary value used while a node is leased
    TmpRemoved,
    // Sources
    Trigger,
    Signal(SignalState),
    Animation(AnimationState),
    EventEmitter,
    // Derived nodes
    Memo(CachedState),
    DerivedAnimation(DerivedAnimationState),
    // Effects (reactions)
    Effect(EffectState),
    InvalidateWidget {
        widget_id: WidgetId,
        phase: WidgetPhase,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetPhase {
    SyncWidget,
    Render,
}

impl NodeType {
    pub fn get_value_ref(&self) -> &dyn Any {
        match self {
            NodeType::TmpRemoved => {
                panic!("Trying to get the value of a node that is currently removed")
            }
            NodeType::Trigger => &(),
            NodeType::Signal(signal_state) => signal_state.value.as_ref(),
            NodeType::Memo(memo_state) => memo_state
                .value
                .as_ref()
                .expect("Memo should have been evaluated before accessed")
                .as_ref(),
            NodeType::Effect(_) => panic!("Cannot get value of an effect"),
            NodeType::Animation(state) => state.inner.value_dyn(),
            NodeType::DerivedAnimation(state) => state.inner.value_dyn(),
            NodeType::EventEmitter => panic!("Cannot get value of eventemitter"),
            NodeType::InvalidateWidget { .. } => panic!("Cannot get value of WidgetSink"),
        }
    }
}

pub mod prelude {
    pub use super::{
        Animated, AnimatedVar, Cached, CanCreate, CanRead, CanWrite, Computed, Effect,
        ReactiveValue, SpringOptions, Trigger, TweenOptions, Var,
    };
}
