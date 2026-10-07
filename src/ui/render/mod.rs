use crate::{
    core::{
        FillRule, ImageData, PathBuilder, PathSegment, Point, Rect, ScaleFactor, Shape, Size,
        TextLayout, Transform, TranslateScale,
    },
    ui::{
        TextContext, WidgetData, Widgets,
        app_state::WidgetMap,
        reactive::{CanRead, ReactiveGraph, ReadContext, ReadScope},
        render::scene::GpuAppearance,
    },
};

mod canvas;
mod draw_style;
mod glyph_cache;
mod gradient_cache;
mod scene;
mod tiles;
mod wgpu_surface;
pub use canvas::{Canvas, CanvasWidget};
pub use draw_style::{DrawStyle, Paint, PaintRef, ShadowKind, ShadowOptions};
pub use glyph_cache::GlyphCache;
pub use scene::GpuScene;
pub use wgpu_surface::WGPUSurface;

use super::{WidgetId, WindowId};

pub fn invalidate_window(widgets: &Widgets, window_id: WindowId) {
    let handle = &widgets.window(window_id).handle;
    handle.invalidate_window()
}

pub fn paint_window(
    widgets: &mut Widgets,
    text_cx: &mut TextContext,
    reactive_graph: &mut ReactiveGraph,
    widget_map: &mut WidgetMap,
    window_id: WindowId,
    dirty_rect: Rect,
) {
    rebuild_scene(widgets, text_cx, reactive_graph, widget_map, window_id);
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

    // There's no point in running the compute shader if there's nothing to render...
    // But maybe we need to clear the texture?
    // Could run an "empty" shader instead of the blit shader, to just clear
    if !window.gpu_scene.is_empty() {
        wgpu_surface.upload_scene(&window.gpu_scene);

        let [dims_x, dims_y] = wgpu_surface.render_tiles_workgroup_count();
        let scene_state = wgpu_surface.scene_state.as_mut().unwrap();
        let surface_state = wgpu_surface.state.as_mut().unwrap();

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&wgpu_surface.render_tiles_program.pipeline);
            compute_pass.set_bind_group(0, &surface_state.render_tiles_bind_group0, &[]);
            compute_pass.set_bind_group(1, &scene_state.render_tiles_bind_group1, &[]);
            compute_pass.dispatch_workgroups(dims_x, dims_y, 1);
        }
    }

    {
        let surface_state = wgpu_surface.state.as_mut().unwrap();
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
        render_pass.set_bind_group(0, &surface_state.blit_bind_group, &[]);
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

fn rebuild_scene(
    widgets: &mut Widgets,
    text_cx: &mut TextContext,
    reactive_graph: &mut ReactiveGraph,
    widget_map: &mut WidgetMap,
    window_id: WindowId,
) {
    let window = &mut widgets.windows[window_id];
    let window_size = window.handle.global_bounds().size();
    let scale_factor = window.handle.scale_factor();
    let mut gpu_scene = std::mem::take(&mut window.gpu_scene);
    gpu_scene.clear();

    let mut roots = vec![window.root_widget];
    roots.extend(window.overlays.iter());

    let mut path_segment_buffer = Vec::new();

    for root_id in roots {
        let mut walker = widgets
            .tree
            .dfs_walker_with_pruning(root_id, |node| !(node.is_overlay() || node.style.hidden));
        while let Some(widget_id) = walker.next(&widgets.tree) {
            let bounds = widgets.tree[widget_id].content_bounds();
            render_node_background(
                &widgets.tree[widget_id],
                &mut gpu_scene,
                window_size,
                scale_factor,
            );

            widget_map[widget_id].render(&mut RenderContext {
                id: widget_id,
                widgets,
                reactive_graph,
                read_scope: ReadScope::Untracked,
                scene: &mut gpu_scene,
                path_segment_buffer: &mut path_segment_buffer,
                scale_factor,
            });

            if let Some(text) = widgets.texts.get_mut(widget_id) {
                let layout = text.get_or_create_layout(text_cx);
                layout.break_all_lines(Some(bounds.width()));
                gpu_scene.draw_text(bounds.top_left(), layout, text_cx.glyph_cache, scale_factor);
            }
        }
    }

    drop(std::mem::replace(
        &mut widgets.windows[window_id].gpu_scene,
        gpu_scene,
    ));
}

fn render_node_background(
    node: &WidgetData,
    gpu_scene: &mut GpuScene,
    window_size: Size,
    scale_factor: ScaleFactor,
) {
    let mut appearance = GpuAppearance::new();
    if let Some(shadow) = node.style.box_shadow {
        appearance.set_shadow(shadow, scale_factor);
    }

    if let Some(background) = &node.style.background {
        let bounds = node.content_bounds();
        appearance.set_fill(PaintRef::from_fill(background, window_size, bounds));
    }

    let line_width = node.layout.border.top * scale_factor.0;
    if let Some(border) = &node.style.border
        && line_width > 0.0
    {
        let bounds = node.global_bounds();
        appearance.set_stroke(
            PaintRef::from_fill(&border.fill, window_size, bounds),
            line_width,
        );
    }

    if !appearance.is_empty() {
        let shape_ref = gpu_scene.add_shape(node.shape());
        let appearance_ref = gpu_scene.add_appearance(appearance);
        gpu_scene.draw(
            shape_ref,
            appearance_ref,
            TranslateScale::scale(scale_factor.0),
        );
    }
}

pub struct RenderContext<'a> {
    pub(super) id: WidgetId,
    pub(super) widgets: &'a mut Widgets,
    pub(super) reactive_graph: &'a mut ReactiveGraph,
    pub(super) read_scope: ReadScope,
    pub(super) path_segment_buffer: &'a mut Vec<PathSegment>,
    pub(super) scene: &'a mut GpuScene,
    pub(super) scale_factor: ScaleFactor,
}

impl<'a> RenderContext<'a> {
    pub fn bounds(&self) -> Rect {
        self.widgets.content_bounds(self.id)
    }

    pub fn has_focus(&self) -> bool {
        self.widgets.has_focus(self.id)
    }

    pub fn has_mouse_capture(&self) -> bool {
        self.widgets.is_pressed(self.id)
    }

    fn current_transform(&self) -> TranslateScale {
        TranslateScale::scale(self.scale_factor.0)
    }

    pub fn fill_path<'b>(
        &mut self,
        mut build_path: impl FnMut(&mut PathBuilder),
        brush: impl Into<PaintRef<'b>>,
        fill_rule: FillRule,
    ) {
        self.path_segment_buffer.clear();
        build_path(&mut PathBuilder::new(self.path_segment_buffer));
        let shape_ref = self.scene.add_path(&self.path_segment_buffer, fill_rule);
        let mut appearance: GpuAppearance = GpuAppearance::new();
        appearance.set_fill(self.current_transform() * brush.into());
        let appearance_ref = self.scene.add_appearance(appearance);
        self.scene
            .draw(shape_ref, appearance_ref, self.current_transform());
    }

    pub fn stroke_path<'b>(
        &mut self,
        mut build_path: impl FnMut(&mut PathBuilder),
        brush: impl Into<PaintRef<'b>>,
        line_width: f32,
    ) {
        self.path_segment_buffer.clear();
        build_path(&mut PathBuilder::new(self.path_segment_buffer));
        let shape_ref = self
            .scene
            .add_path(&self.path_segment_buffer, FillRule::NonZero);
        let mut appearance = GpuAppearance::new();
        appearance.set_stroke(brush.into(), line_width * self.scale_factor.0);
        let appearance_ref = self.scene.add_appearance(appearance);
        self.scene.draw(
            shape_ref,
            appearance_ref,
            TranslateScale::scale(self.scale_factor.0),
        );
    }

    pub fn fill<'b>(&mut self, shape: impl Into<Shape>, brush: impl Into<PaintRef<'b>>) {
        let shape_ref = self.scene.add_shape(shape.into());
        let mut appearance = GpuAppearance::new();
        appearance.set_fill(self.current_transform() * brush.into());
        let appearance_ref = self.scene.add_appearance(appearance);
        self.scene.draw(
            shape_ref,
            appearance_ref,
            TranslateScale::scale(self.scale_factor.0),
        );
    }

    pub fn stroke<'d>(
        &mut self,
        shape: impl Into<Shape>,
        brush: impl Into<PaintRef<'d>>,
        line_width: f32,
    ) {
        let shape_ref = self.scene.add_shape(shape.into());
        let mut appearance = GpuAppearance::new();
        appearance.set_stroke(self.current_transform() * brush.into(), line_width);
        let appearance_ref = self.scene.add_appearance(appearance);
        self.scene.draw(
            shape_ref,
            appearance_ref,
            TranslateScale::scale(self.scale_factor.0),
        );
    }

    pub fn draw_line<'b>(
        &mut self,
        p0: Point,
        p1: Point,
        brush: impl Into<PaintRef<'b>>,
        line_width: f32,
    ) {
        self.stroke_path(
            |path| {
                path.move_to(p0).line_to(p1);
            },
            brush.into(),
            line_width,
        );
    }

    pub fn draw_lines<'c>(
        &mut self,
        points: &[Point],
        brush: impl Into<PaintRef<'c>>,
        line_width: f32,
    ) {
        if points.len() > 1 {
            self.stroke_path(
                |mut path| {
                    let mut it = points.iter().copied();
                    path = path.move_to(it.next().unwrap());
                    for point in it {
                        path = path.line_to(point);
                    }
                },
                brush.into(),
                line_width,
            );
        }
    }

    pub fn draw_bitmap(&mut self, source: &ImageData, rect: impl Into<Rect>) {
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

impl<'s> CanRead<'s> for RenderContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: self.read_scope,
            current_widget: Some(self.id),
        }
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
