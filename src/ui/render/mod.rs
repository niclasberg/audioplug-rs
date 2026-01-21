use crate::{
    core::{
        Color, ColorMap, Ellipse, FillRule, Path, Point, Rect, RoundedRect, ShadowOptions, Size,
        Vec2, Vec2f,
    },
    platform,
    ui::render::gpu_scene::GpuFill,
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

use super::{AppState, WidgetId, WindowId};
pub use platform::TextLayout;

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

pub fn paint_window(app_state: &mut AppState, window_id: WindowId, dirty_rect: Rect) {
    let window = &mut app_state.windows[window_id];
    let wgpu_surface = &mut window.wgpu_surface;
    let scale_factor = window.handle.scale_factor().0;

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

    let mut gpu_scene = GpuScene::new();
    {
        let rect = gpu_scene.add_rect(Rect {
            left: 10.0,
            top: 10.0,
            right: 150.0,
            bottom: 200.0,
        });
        let rect2 = gpu_scene.add_rect(Rect {
            left: 10.0,
            top: 310.0,
            right: 150.0,
            bottom: 500.0,
        });
        let rounded_rect = gpu_scene.add_rounded_rect(
            RoundedRect::new(
                Rect {
                    left: 50.3,
                    top: 100.2,
                    right: 500.0,
                    bottom: 400.3,
                },
                Size::new(40.0, 30.0),
            )
            .scale(scale_factor),
        );
        let path = gpu_scene.add_path(
            &Path::new()
                .move_to(Point::new(100.0, 100.0))
                .line_to(Point::new(100.0, 800.0))
                .line_to(Point::new(800.0, 800.0))
                .line_to(Point::new(700.0, 400.0))
                .close_path(),
            FillRule::NonZero,
        );
        let ellipse = gpu_scene.add_ellipse(
            Ellipse::from_rectangle(Rect {
                left: 650.3,
                top: 200.2,
                right: 900.0,
                bottom: 300.3,
            })
            .scale(scale_factor),
        );
        let drop_shadow = GpuFill::Shadow(ShadowOptions {
            radius: 25.0,
            offset: Vec2::splat(10.0),
            color: Color::BLACK.with_alpha(0.7),
            kind: crate::core::ShadowKind::DropShadow,
        });

        gpu_scene.fill_shape(rect, drop_shadow.clone());
        gpu_scene.fill_shape(rect, GpuFill::Solid(Color::RED));
        gpu_scene.fill_shape(
            path,
            GpuFill::LinearGradient {
                start: Vec2f { x: 100.0, y: 100.0 },
                end: Vec2f { x: 800.0, y: 800.0 },
                color_stops: ColorMap::new([]),
            },
        );
        gpu_scene.fill_shape(ellipse, drop_shadow.clone());
        gpu_scene.fill_shape(ellipse, GpuFill::Solid(Color::RED));

        gpu_scene.fill_shape(rounded_rect, drop_shadow.clone());
        gpu_scene.fill_shape(
            rounded_rect,
            GpuFill::Solid(Color::CHAMOISEE.with_alpha(0.7)),
        );
        gpu_scene.fill_shape(
            rect2,
            GpuFill::RadialGradient {
                center: Vec2f { x: 75.0, y: 400.0 },
                radius: 80.0,
                color_stops: ColorMap::new([]),
            },
        )
    }

    let mut encoder = wgpu_surface
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("AudioPlug command encoder"),
        });
    wgpu_surface.upload_scene(&gpu_scene);

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
