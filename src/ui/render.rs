use crate::{
    core::{Point, Rect, ShadowKind, Transform},
    platform,
};

mod canvas;
mod scene;
pub use canvas::{Canvas, CanvasContext, CanvasWidget};
pub use scene::Scene;

use super::{AppState, WidgetId, WindowId};
pub use platform::TextLayout;
pub use platform::{Brush, BrushRef, LinearGradient, RadialGradient};
pub use platform::{PathGeometry, PathGeometryBuilder, Shape, ShapeRef};

pub fn render_window(app_state: &mut AppState, window_id: WindowId) {
    app_state.with_id_buffer_mut(move |app_state, widgets_to_render| {
        widgets_to_render.extend(
            app_state
                .window_mut(window_id)
                .widgets_needing_render
                .drain(..),
        );

        for widget_id in widgets_to_render {
            let mut cx = RenderContext {
                id: *widget_id,
                app_state,
            };
            cx.render_current_widget();
        }
    });
}

pub fn invalidate_window(app_state: &AppState, window_id: WindowId) {
    let handle = &app_state.window(window_id).handle;
    handle.invalidate_window()
}

pub fn invalidate_widget(app_state: &AppState, widget_id: WidgetId) {
    let bounds = app_state.widget_data[widget_id].global_bounds();
    let window_id = app_state.get_window_id_for_widget(widget_id);
    let handle = &app_state.window(window_id).handle;
    handle.invalidate(bounds);
}

pub struct RenderContext<'a> {
    id: WidgetId,
    app_state: &'a mut AppState,
}

impl RenderContext<'_> {
    pub fn local_bounds(&self) -> Rect {
        self.app_state.widget_data_ref(self.id).local_bounds()
    }

    pub fn global_bounds(&self) -> Rect {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    pub fn content_bounds(&self) -> Rect {
        self.app_state.widget_data_ref(self.id).content_bounds()
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.app_state.widget_has_captured_mouse(self.id)
    }

    fn render_current_widget(&mut self) {
        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        let scene = widget.render(self);
        self.app_state.widgets.insert(self.id, widget);
        self.app_state.widget_data_mut(self.id).scene = scene;
        invalidate_widget(self.app_state, self.id);
    }
}
