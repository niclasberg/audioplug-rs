use crate::{
    core::{Point, Rectangle, ShadowKind, Transform},
    platform,
};

mod canvas;
mod graphics_context;
use super::{AppState, WidgetId, WindowId};
pub use canvas::{Canvas, CanvasContext, CanvasWidget};
pub use platform::TextLayout;
pub use platform::{Brush, BrushRef, LinearGradient, RadialGradient};
pub use platform::{PathGeometry, PathGeometryBuilder, Shape, ShapeRef};

pub fn render_window(
    app_state: &mut AppState,
    window_id: WindowId,
    renderer: platform::RendererRef<'_>,
) {
    println!("Render, dirty rect: {:?}", renderer.dirty_rect());
    app_state.with_id_buffer_mut(move |app_state, overlays| {
        overlays.extend(app_state.window(window_id).overlays.iter());

        // Root
        let mut cx = RenderContext {
            id: app_state.window(window_id).root_widget,
            app_state,
            renderer,
        };
        cx.render_current_widget();

        // Overlays
        for overlay_id in overlays.iter() {
            cx.id = *overlay_id;
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

pub struct RenderContext<'a, 'b> {
    id: WidgetId,
    app_state: &'a mut AppState,
    renderer: platform::RendererRef<'b>,
}

impl RenderContext<'_, '_> {
    pub fn local_bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    pub fn content_bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).content_bounds()
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.app_state.widget_has_captured_mouse(self.id)
    }

    pub fn fill<'c, 'd>(&mut self, shape: impl Into<ShapeRef<'c>>, brush: impl Into<BrushRef<'d>>) {
        self.renderer.fill_shape(shape.into(), brush.into());
    }

    pub fn stroke<'c, 'd>(
        &mut self,
        shape: impl Into<ShapeRef<'c>>,
        brush: impl Into<BrushRef<'d>>,
        line_width: f32,
    ) {
        self.renderer
            .stroke_shape(shape.into(), brush.into(), line_width);
    }

    pub fn draw_line<'c>(
        &mut self,
        p0: Point,
        p1: Point,
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        self.renderer.draw_line(p0, p1, brush.into(), line_width)
    }

    pub fn draw_lines<'c>(
        &mut self,
        points: &[Point],
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        let brush = brush.into();
        for p in points.windows(2) {
            self.renderer.draw_line(p[0], p[1], brush, line_width)
        }
    }

    pub fn draw_bitmap(&mut self, source: &platform::Bitmap, rect: impl Into<Rectangle>) {
        self.renderer.draw_bitmap(source, rect.into())
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn use_clip(
        &mut self,
        rect: impl Into<Rectangle>,
        f: impl FnOnce(&mut RenderContext<'_, '_>),
    ) {
        self.renderer.save();
        self.renderer.clip(rect.into());
        f(self);
        self.renderer.restore();
    }

    pub fn transform(&mut self, transform: impl Into<Transform>) {
        self.renderer.transform(transform.into());
    }

    pub(crate) fn render_current_widget(&mut self) {
        {
            let widget_data = self.app_state.widget_data_ref(self.id);
            if widget_data.is_hidden()
                || !widget_data
                    .global_bounds()
                    .intersects(&self.renderer.dirty_rect())
            {
                return;
            }

            let border_color = widget_data.style.border_color;
            let line_width = widget_data.layout.border.top;
            let shape = widget_data.shape();

            if let Some(shadow) = &widget_data.style.box_shadow {
                if shadow.kind == ShadowKind::DropShadow {
                    self.renderer.draw_shadow((&shape).into(), *shadow);
                }
            }

            if let Some(background) = &widget_data.style.background {
                self.renderer.fill_shape((&shape).into(), background.into());
            }

            if let Some(border_color) = border_color {
                self.stroke(&shape, border_color, line_width);
            }
        }

        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        widget.render(self);
        self.app_state.widgets.insert(self.id, widget);

        {
            let widget_data = self.app_state.widget_data_ref(self.id);
            if let Some(shadow) = widget_data.style.box_shadow {
                if shadow.kind == ShadowKind::InnerShadow {
                    self.renderer
                        .draw_shadow((&widget_data.shape()).into(), shadow);
                }
            }
        }
    }

    pub fn render_children(&mut self) {
        let old_id = self.id;
        let ids = self
            .app_state
            .widget_data
            .get(self.id)
            .expect("Could not find widget")
            .children
            .clone();
        for id in ids {
            // Overlay children are handled at root level
            if self.app_state.widget_data[id].is_overlay() {
                continue;
            }
            if self.app_state.widget_data[id].is_hidden() {
                continue;
            }
            self.id = id;
            self.render_current_widget();
        }
        self.id = old_id;
    }

    pub fn dirty_rect(&self) -> Rectangle {
        self.renderer.dirty_rect()
    }
}
