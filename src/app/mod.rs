mod accessor;
mod animation;
mod app_state;
mod brush;
mod clipboard;
pub mod diff;
mod effect;
mod event_handling;
mod host_handle;
mod layout;
mod memo;
mod overlay;
mod param;
mod render;
mod signal;
mod signal_range;
mod signal_vec;
mod runtime;
mod trigger;
mod traits;
mod view;
mod view_sequence;
mod widget;
mod widget_data;
mod widget_ref;
mod window;

use std::{cell::RefCell, rc::Rc};

pub use accessor::Accessor;
pub use animation::{AnimationContext, Animated};
pub(crate) use app_state::AppState;
pub use brush::{Brush, BrushRef, LinearGradient, RadialGradient};
pub use effect::{Effect, EffectState};
pub use event_handling::{EventContext, MouseEventContext, handle_window_event};
pub use host_handle::HostHandle;
pub use layout::{LayoutContext, layout_window};
pub use memo::{Memo, MemoContext};
pub use param::{ParamContext, ParamEditor, ParamSignal};
pub use render::{RenderContext, render_window, invalidate_window};
pub use runtime::*;
pub use signal::{Signal, ReadSignal};
pub use traits::*;
pub use trigger::{Trigger, DependentField};
pub use view::*;
pub use view_sequence::*;
pub use widget::{EventStatus, StatusChange, Widget};
pub use widget_ref::{WidgetRef, WidgetMut};
pub use widget_data::{WidgetData, WidgetId, WidgetFlags};
pub use window::Window;
#[cfg(target_os  ="macos")]
pub(crate) use window::MyHandler;

use crate::{param::ParameterMap, platform};

slotmap::new_key_type! {
    pub struct NodeId;
}

slotmap::new_key_type! {
    pub struct WindowId;
}

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
            state
        }
    }

    pub fn run(&mut self) {
        self.native.run()
    }
}

mod prelude {

}