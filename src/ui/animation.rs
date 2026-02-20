use crate::{
    AnimationFrame,
    ui::{AppState, WidgetId, Widgets},
};

/// Should be called when the animation timer for a window ticks.
/// Steps all animations that have been enqueued for window.
pub(super) fn drive_animations(app_state: &mut AppState, animation_frame: AnimationFrame) {
    for widget_id in app_state.widgets.take_requested_animations() {
        if let Some(mut widget) = app_state.widgets.lease_widget(widget_id) {
            widget.animation_frame(
                animation_frame,
                &mut AnimationContext {
                    id: widget_id,
                    widgets: &mut app_state.widgets,
                },
            );
            app_state.widgets.unlease_widget(widget);
        }
    }

    app_state
        .reactive_graph
        .drive_animations(&mut app_state.widgets, &mut app_state.task_queue);

    app_state.run_effects();
}

pub struct AnimationContext<'a> {
    id: WidgetId,
    widgets: &'a mut Widgets,
}

impl AnimationContext<'_> {
    pub fn has_focus(&self) -> bool {
        self.widgets.widget_has_focus(self.id)
    }

    pub fn request_render(&mut self) {
        self.widgets.invalidate_widget(self.id);
    }

    pub fn request_animation(&mut self) {
        self.widgets.request_animation(self.id);
    }

    pub fn request_layout(&mut self) {
        self.widgets.request_layout(self.id);
    }
}
