mod accessor;
mod animation;
mod cached;
pub(super) mod diff;
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

pub use accessor::{Accessor, Computed};
pub use animation::{Animated, AnimatedFn, Animation, Easing, SpringOptions, TweenOptions};

pub use cached::{Cached, CachedContext};
pub(super) use effect::{BindingFn, EffectFn};
pub use effect::{Effect, EffectContext, EffectState, WatchContext};
pub(super) use event_channel::HandleEventFn;
pub use event_channel::{EventChannel, EventReceiver, create_event_channel};
pub use read_signal::ReadSignal;
pub use readable::*;
pub use runtime::*;
pub use trigger::Trigger;
pub use var::Var;

slotmap::new_key_type! {
    pub struct NodeId;
}
