use crate::{core::{Point, Rectangle, Shape, Transform}, platform, text::TextLayout};

use super::{brush::BrushRef, AppState, WidgetId, WindowId};

pub fn render_window(app_state: &mut AppState, window_id: WindowId, renderer: platform::RendererRef<'_>) {
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
    renderer: platform::RendererRef<'b>
}

impl<'a, 'b> RenderContext<'a, 'b> {
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

    pub fn fill<'d>(&mut self, shape: impl Into<Shape>, brush: impl Into<BrushRef<'d>>) {
        do_fill(&mut self.renderer, shape.into(), brush.into());
    }

    pub fn stroke<'d>(&mut self, shape: impl Into<Shape>, brush: impl Into<BrushRef<'d>>, line_width: f32) {
        let brush = brush.into();
        match shape.into() {
            Shape::Rect(rect) => self.renderer.draw_rectangle(rect, brush, line_width),
            Shape::Rounded(rect) => self.renderer.draw_rounded_rectangle(rect, brush, line_width),
            Shape::Ellipse(ell) => self.renderer.draw_ellipse(ell, brush, line_width),
            Shape::Line { p0, p1 } => self.renderer.draw_line(p0, p1, brush, line_width)
        }
    }

    pub fn draw_bitmap(&mut self, source: &platform::NativeImage, rect: impl Into<Rectangle>) {
        self.renderer.draw_bitmap(source, rect.into())
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn use_clip(&mut self, rect: impl Into<Rectangle>, f: impl FnOnce(&mut RenderContext<'_, '_>)) {
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

            if let Some(background) = &widget_data.style.background {
                do_fill(&mut self.renderer, shape, background.into());
            }

            if let Some(border_color) = border_color {
                self.stroke(shape, border_color, line_width);
            }
        }

        let mut widget = self.app_state.widgets.remove(self.id).unwrap();
        widget.render(self);
        self.app_state.widgets.insert(self.id, widget);
    }

    pub fn render_children(&mut self) {
        let old_id = self.id;
        let ids = self.app_state.widget_data.get(self.id).expect("Could not find widget").children.clone();
        for id in ids {
            self.id = id;
            self.render_current_widget();
        }
        self.id = old_id;
    }
}

fn do_fill(renderer: platform::RendererRef, shape: Shape, brush: BrushRef) {
    match shape {
        Shape::Rect(rect) => renderer.fill_rectangle(rect, brush),
        Shape::Rounded(rect) => renderer.fill_rounded_rectangle(rect, brush),
        Shape::Ellipse(ell) => renderer.fill_ellipse(ell, brush),
        Shape::Line { p0, p1 } => renderer.draw_line(p0, p1, brush, 1.0)
    }
}