use crate::{
    core::{Brush, BrushRef, Point, Rect, ShadowKind, ShapeRef, Transform, Vec2f},
    platform,
    ui::{Widgets, reactive::ReactiveGraph, render::gpu_scene::GpuFill},
};

mod canvas;
mod gpu_scene;
mod gradient_cache;
mod scene;
mod tiles;
mod wgpu_surface;
pub use canvas::{Canvas, CanvasContext, CanvasWidget};
pub use gpu_scene::GpuScene;
pub use scene::Scene;
pub use wgpu_surface::WGPUSurface;

use super::{WidgetId, WindowId};
pub use platform::TextLayout;

pub fn invalidate_window(widgets: &Widgets, window_id: WindowId) {
    let handle = &widgets.window(window_id).handle;
    handle.invalidate_window()
}

pub fn paint_window(widgets: &mut Widgets, window_id: WindowId, dirty_rect: Rect) {
    rebuild_scene(widgets, window_id);
    let window = widgets.window_mut(window_id);
    let wgpu_surface = &mut window.wgpu_surface;

    println!("Paint window, dirty rect: {dirty_rect:?}");
    wgpu_surface.configure_if_needed(window.handle.physical_size());
    if !wgpu_surface.is_configured {
        return;
    }

    let surface_texture = wgpu_surface
        .surface
        .get_current_texture()
        .expect("Unable to get surface texture");
    let texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu_surface.surface_format.add_srgb_suffix()),
            ..Default::default()
        });

    let mut encoder = wgpu_surface
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("AudioPlug command encoder"),
        });
    wgpu_surface.upload_scene(&window.gpu_scene);

    let dims = wgpu_surface.render_tiles_workgroup_count();
    let state = wgpu_surface.state.as_mut().unwrap();

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        compute_pass.set_pipeline(&wgpu_surface.render_tiles_program.pipeline);
        compute_pass.set_bind_group(0, &state.render_tiles_bind_group0, &[]);
        compute_pass.set_bind_group(1, &wgpu_surface.render_tiles_bind_group1, &[]);
        compute_pass.dispatch_workgroups(dims.width, dims.height, 1);
    }

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Blit the texture to the render target
        render_pass.set_pipeline(&wgpu_surface.blit_program.pipeline);
        render_pass.set_bind_group(0, &state.blit_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    wgpu_surface.queue.submit(std::iter::once(encoder.finish()));

    surface_texture.present();

    /*
    overlays.extend(app_state.window(window_id).overlays.iter());

        // Root
        let mut cx = RenderContext {
            id: app_state.window(window_id).root_widget,
            app_state,
        };
        cx.render_current_widget();

        // Overlays
        for overlay_id in overlays.iter() {
            cx.id = *overlay_id;
            cx.render_current_widget();
        }
         */
}

fn rebuild_scene(widgets: &mut Widgets, window_id: WindowId) {
    let window = &mut widgets.windows[window_id];
    let scale_factor = window.handle.scale_factor().0;
    let gpu_scene = &mut window.gpu_scene;
    gpu_scene.clear();

    let mut roots = vec![window.root_widget];
    roots.extend(window.overlays.iter());

    for root_id in roots {
        let mut walker = widgets.tree.dfs_walker(root_id);
        while let Some(widget_id) = walker.next(&widgets.tree) {
            let node = &widgets.tree[widget_id];
            if node.is_overlay() || node.style.hidden {
                continue;
            }

            let shape = node.shape().scale(scale_factor);
            let mut inner_shape_ref = None;
            let shadow = node.style.box_shadow;
            if let Some(shadow) = shadow
                && shadow.kind == ShadowKind::DropShadow
            {
                let shape_ref = gpu_scene.add_primitive_shape(shape);
                gpu_scene.fill_shape(shape_ref, GpuFill::Shadow(shadow));
                inner_shape_ref = Some(shape_ref);
            }

            if let Some(background) = &node.style.background {
                let shape_ref =
                    inner_shape_ref.get_or_insert_with(|| gpu_scene.add_primitive_shape(shape));
                let fill = match background {
                    Brush::Solid(color) => GpuFill::Solid(*color),
                    Brush::LinearGradient(linear_gradient) => GpuFill::LinearGradient {
                        start: Vec2f::ZERO,
                        end: Vec2f::ZERO,
                        color_stops: linear_gradient.color_map.clone(),
                    },
                };
                gpu_scene.fill_shape(*shape_ref, fill);
            }

            let line_width = node.layout.border.top as f64 * scale_factor;
            if let Some(border_color) = node.style.border_color
                && line_width > 0.0
            {
                let shape_ref = gpu_scene.add_primitive_shape(shape.inflate(line_width / 2.0));
                gpu_scene.fill_shape(
                    shape_ref,
                    GpuFill::Stroke {
                        color: border_color,
                        width: line_width as _,
                    },
                );
            }
        }
    }
}

pub struct RenderContext<'a> {
    id: WidgetId,
    pub(super) widgets: &'a mut Widgets,
    pub(super) reactive_graph: &'a mut ReactiveGraph,
    scene: &'a mut GpuScene,
}

impl<'a> RenderContext<'a> {
    pub fn local_bounds(&self) -> Rect {
        self.widgets.local_bounds(self.id)
    }

    pub fn global_bounds(&self) -> Rect {
        self.widgets.global_bounds(self.id)
    }

    pub fn content_bounds(&self) -> Rect {
        self.widgets.content_bounds(self.id)
    }

    pub fn has_focus(&self) -> bool {
        self.widgets.has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.widgets.has_mouse_capture(self.id)
    }

    pub fn fill<'b>(&mut self, shape: impl Into<ShapeRef<'b>>, brush: impl Into<Brush>) {
        //self.renderer.fill_shape(shape.into(), brush.into());
    }

    pub fn stroke<'c, 'd>(
        &mut self,
        shape: impl Into<ShapeRef<'c>>,
        brush: impl Into<BrushRef<'d>>,
        line_width: f32,
    ) {
        //self.renderer.stroke_shape(shape.into(), brush.into(), line_width);
    }

    pub fn draw_line<'c>(
        &mut self,
        p0: Point,
        p1: Point,
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        //self.renderer.draw_line(p0, p1, brush.into(), line_width)
    }

    pub fn draw_lines<'c>(
        &mut self,
        points: &[Point],
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        /*let brush = brush.into();
        for p in points.windows(2) {
            self.renderer.draw_line(p[0], p[1], brush, line_width)
        }*/
    }

    pub fn draw_bitmap(&mut self, source: &crate::platform::Bitmap, rect: impl Into<Rect>) {
        //self.renderer.draw_bitmap(source, rect.into())
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        //self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn use_clip(&mut self, rect: impl Into<Rect>, f: impl FnOnce(&mut Self)) {
        /*self.renderer.save();
        self.renderer.clip(rect.into());
        f(self);
        self.renderer.restore();*/
    }

    pub fn transform(&mut self, transform: impl Into<Transform>) {
        //self.renderer.transform(transform.into());
    }
}

/*


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
*/
