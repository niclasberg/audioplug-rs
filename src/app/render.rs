use crate::{
    core::{Point, Rectangle, Transform},
    platform,
};

mod canvas;
mod text;
use super::{AppState, WidgetId, WindowId};
pub use canvas::{Canvas, CanvasContext, CanvasWidget};
pub use platform::{Brush, BrushRef, LinearGradient, RadialGradient};
pub use platform::{PathGeometry, PathGeometryBuilder, Shape, ShapeRef};
pub use text::TextLayout;

pub fn render_window(
    app_state: &mut AppState,
    window_id: WindowId,
    renderer: platform::RendererRef<'_>,
) {
    println!("Render");
    let widget_id = app_state.window(window_id).root_widget;
    let mut ctx = RenderContext {
        id: widget_id,
        app_state,
        renderer,
    };
    ctx.render_current_widget();
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
        fill_shape(self.renderer, shape.into(), brush.into());
    }

    pub fn stroke<'c, 'd>(
        &mut self,
        shape: impl Into<ShapeRef<'c>>,
        brush: impl Into<BrushRef<'d>>,
        line_width: f32,
    ) {
        stroke_shape(self.renderer, shape.into(), brush.into(), line_width);
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
            if widget_data.is_hidden() {
                return;
            }

            let border_color = widget_data.style.border_color;
            let line_width = widget_data.layout.border.top;
            let shape = widget_data.shape();

            if let Some(shadow) = &widget_data.style.box_shadow {
                self.renderer.draw_shadow((&shape).into(), *shadow);
            }

            if let Some(background) = &widget_data.style.background {
                fill_shape(self.renderer, (&shape).into(), background.into());
            }

            if let Some(border_color) = border_color {
                self.stroke(&shape, border_color, line_width);
            }
        }

        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        widget.render(self);
        self.app_state.widgets.insert(self.id, widget);
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
            self.id = id;
            self.render_current_widget();
        }
        self.id = old_id;
    }
}

pub fn fill_shape(renderer: platform::RendererRef, shape: ShapeRef, brush: BrushRef) {
    match shape {
        ShapeRef::Rect(rect) => renderer.fill_rectangle(rect, brush),
        ShapeRef::Rounded(rect) => renderer.fill_rounded_rectangle(rect, brush),
        ShapeRef::Ellipse(ell) => renderer.fill_ellipse(ell, brush),
        ShapeRef::Geometry(geometry) => renderer.fill_geometry(&geometry.0, brush),
    }
}

pub fn stroke_shape(
    renderer: platform::RendererRef,
    shape: ShapeRef,
    brush: BrushRef,
    line_width: f32,
) {
    match shape {
        ShapeRef::Rect(rect) => renderer.draw_rectangle(rect, brush, line_width),
        ShapeRef::Rounded(rect) => renderer.draw_rounded_rectangle(rect, brush, line_width),
        ShapeRef::Ellipse(ell) => renderer.draw_ellipse(ell, brush, line_width),
        ShapeRef::Geometry(geometry) => renderer.draw_geometry(&geometry.0, brush, line_width),
    }
}
