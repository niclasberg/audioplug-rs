use crate::{core::{Color, Point, Rectangle, Shape, Transform}, platform, text::TextLayout};

use super::{AppState, WidgetId, WindowId};

pub fn render_window(app_state: &mut AppState, window_id: WindowId, renderer: &mut platform::RendererRef<'_>) {
    let widget_id = app_state.window(window_id).root_widget;
    let mut ctx = RenderContext {
        id: widget_id,
        app_state,
        renderer,
    };
    ctx.render_current_widget();
}

pub fn invalidate_window(app_state: &mut AppState, window_id: WindowId) {
    let handle = &app_state.window(window_id).handle;
    handle.invalidate(handle.global_bounds())
}

pub struct RenderContext<'a, 'b, 'c> {
    id: WidgetId, 
    app_state: &'a mut AppState,
    renderer: &'b mut platform::RendererRef<'c>
}

impl<'a, 'b, 'c> RenderContext<'a, 'b, 'c> {
    pub fn local_bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).local_bounds()
    }

    pub fn global_bounds(&self) -> Rectangle {
        self.app_state.widget_data_ref(self.id).global_bounds()
    }

    pub fn has_focus(&self) -> bool {
        self.app_state.widget_has_focus(self.id)
    }

    pub fn fill(&mut self, shape: impl Into<Shape>, color: impl Into<Color>) {
		let color = color.into();
        match shape.into() {
            Shape::Rect(rect) => 
                self.renderer.fill_rectangle(rect, color),
            Shape::RoundedRect { rect, corner_radius } => 
                self.renderer.fill_rounded_rectangle(rect, corner_radius, color),
            Shape::Ellipse { center, radii } => 
                self.renderer.fill_ellipse(center, radii, color),
            Shape::Line { p0, p1 } =>
                self.renderer.draw_line(p0, p1, color, 1.0)
        }
    }

    pub fn stroke(&mut self, shape: impl Into<Shape>, color: impl Into<Color>, line_width: f32) {
        match shape.into() {
            Shape::Rect(rect) => 
                self.renderer.draw_rectangle(rect, color.into(), line_width),
            Shape::RoundedRect { rect, corner_radius } => 
                self.renderer.draw_rounded_rectangle(
                    rect, 
                    corner_radius, 
                    color.into(), 
                    line_width),
            Shape::Ellipse { center, radii } => self.renderer.draw_ellipse(center, radii, color.into(), line_width),
            Shape::Line { p0, p1 } => self.renderer.draw_line(p0, p1, color.into(), line_width)
        }
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn use_clip(&mut self, rect: impl Into<Rectangle>, f: impl FnOnce(&mut RenderContext<'_, '_, 'c>)) {
        self.renderer.save();
        self.renderer.clip(rect.into());
        f(self);
        self.renderer.restore();
    }

    pub fn transform(&mut self, transform: impl Into<Transform>) {
        self.renderer.transform(transform.into());
    }

    pub(crate) fn render_current_widget(&mut self) {
        let widget = self.app_state.widgets.get(self.id).unwrap();
        widget.render(self);
    }

    pub fn render_children(&mut self) {
        let old_id = self.id;
        let ids = self.app_state.widget_data.get(self.id).expect("Could not find widget").children;
        for id in ids {
            self.id = id;
            self.render_current_widget();
        }
        self.id = old_id;
    }
}