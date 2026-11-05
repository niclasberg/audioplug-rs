use crate::{
    AnimationFrame,
    ui::{AppState, UiState, WidgetId},
};

/// Should be called when the animation timer for a window ticks.
/// Steps all animations that have been enqueued for window.
pub(super) fn drive_animations(app_state: &mut AppState, animation_frame: AnimationFrame) {
    app_state.ui.drive_animations();
    app_state.run_effects_and_layout();
}
