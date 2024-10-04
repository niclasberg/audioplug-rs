use crate::AnimationFrame;

use super::{AppState, WidgetId, WindowId};

pub(super) fn drive_animations(app_state: &mut AppState, window_id: WindowId, animation_frame: AnimationFrame) {
    let requested_animations = std::mem::take(&mut app_state.window_mut(window_id).requested_animations);
    for widget_id in requested_animations {

    }
}

pub fn request_animation_frame(app_state: &mut AppState, widget_id: WidgetId) {
    let window_id = app_state.get_window_id_for_widget(widget_id);
    app_state.window_mut(window_id).requested_animations.insert(widget_id);
}

pub struct AnimationContext {

}

