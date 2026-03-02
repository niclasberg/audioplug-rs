mod animation;
mod cached;
mod computed;
mod contexts;
mod effect;
mod event_channel;
mod param;
mod reactive_value;
mod read_signal;
mod runtime;
mod signal_vec;
mod spring;
mod trigger;
mod tween;
mod var;
mod widget_status;

pub use animation::{Animated, AnimatedVar, Animation, Easing, SpringOptions, TweenOptions};
pub use cached::{Cached, CachedContext};
pub use computed::Computed;
pub use contexts::*;
pub use effect::{Effect, EffectContext, WatchContext};
pub(super) use effect::{EffectFn, EffectState, WatchFn};
pub(super) use event_channel::HandleEventFn;
pub use event_channel::{EventChannel, EventReceiver, create_event_channel};
pub use param::ParamSetter;
pub use reactive_value::ReactiveValue;
pub use read_signal::ReadSignal;
pub use runtime::{Owner, ReactiveGraph, ReadScope};
pub use trigger::Trigger;
pub use var::Var;
pub use widget_status::{CLICKED_STATUS, FOCUS_STATUS, WidgetStatusFlags};

slotmap::new_key_type! {
    pub struct NodeId;
}

pub mod prelude {
    pub use super::{
        Animated, AnimatedVar, Cached, CanCreate, CanRead, CanWrite, Computed, Effect,
        ReactiveValue, SpringOptions, Trigger, TweenOptions, Var,
    };
}
