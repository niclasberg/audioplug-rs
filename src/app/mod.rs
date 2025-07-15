mod accessor;
mod animation;
mod app_state;
mod clipboard;
mod diff;
mod effect;
mod event_channel;
mod event_handling;
mod host_handle;
mod layout;
mod memo;
mod overlay;
mod param;
mod read_signal;
mod readable;
mod render;
mod runtime;
mod signal;
mod signal_vec;
mod trigger;
mod view;
mod view_sequence;
mod widget;
mod widget_data;
mod widget_ref;
mod window;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
};

pub use accessor::{Accessor, Computed};
pub use animation::{
    Animated, AnimatedFn, Animation, AnimationContext, Easing, SpringOptions, TweenOptions,
};
pub(crate) use app_state::AppState;
pub use effect::{Effect, EffectContext, EffectState, WatchContext};
pub use event_channel::{create_event_channel, EventChannel, EventReceiver};
pub use event_handling::{handle_window_event, CallbackContext, EventContext, MouseEventContext};
use fxhash::FxBuildHasher;
pub use host_handle::HostHandle;
use indexmap::IndexSet;
pub use layout::{layout_window, LayoutContext};
pub use memo::{Memo, MemoContext};
pub use overlay::{OverlayAnchor, OverlayOptions};
pub use param::{ParamContext, ParamSetter};
pub use read_signal::ReadSignal;
pub use readable::*;
pub use render::{
    invalidate_window, render_window, Brush, BrushRef, Canvas, CanvasContext, CanvasWidget,
    LinearGradient, PathGeometry, PathGeometryBuilder, RadialGradient, RenderContext, Shape,
    ShapeRef, TextLayout,
};
pub use runtime::*;
pub use signal::Signal;
pub use signal_vec::SignalVec;
pub use trigger::Trigger;
pub use view::*;
pub use view_sequence::*;
pub use widget::{EventStatus, StatusChange, Widget, WrappedWidget};
pub use widget_data::{WidgetData, WidgetFlags, WidgetId};
pub use widget_ref::{WidgetMut, WidgetRef};
#[cfg(target_os = "macos")]
pub(crate) use window::MyHandler;
pub use window::Window;

use crate::{param::ParameterMap, platform};

slotmap::new_key_type! {
    pub struct NodeId;
}

slotmap::new_key_type! {
    pub struct WindowId;
}

type FxHashSet<K> = HashSet<K, FxBuildHasher>;
type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

pub struct TypedWidgetId<W: Widget + ?Sized> {
    pub id: WidgetId,
    _phantom: PhantomData<fn() -> W>,
}

impl<W: Widget + ?Sized> TypedWidgetId<W> {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn into_any_widget_id(self) -> AnyWidgetId {
        AnyWidgetId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

pub type AnyWidgetId = TypedWidgetId<dyn Widget>;

impl<T: Widget + ?Sized> Clone for TypedWidgetId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Widget + ?Sized> Copy for TypedWidgetId<T> {}

pub struct App {
    native: platform::Application,
    pub(crate) state: Rc<RefCell<AppState>>,
}

impl App {
    pub fn new() -> Self {
        let parameters = ParameterMap::new(());
        Self::new_with_app_state(Rc::new(RefCell::new(AppState::new(parameters))))
    }

    pub fn new_with_app_state(state: Rc<RefCell<AppState>>) -> Self {
        Self {
            native: platform::Application::new(),
            state,
        }
    }

    pub fn run(&mut self) {
        self.native.run()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

mod prelude {}
