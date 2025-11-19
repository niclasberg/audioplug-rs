use crate::{core::Rect, platform};

mod canvas;
mod gpu_scene;
mod gradient_cache;
mod scene;
mod tiles;
mod wgpu_surface;
pub use canvas::{Canvas, CanvasContext, CanvasWidget};
use pollster::FutureExt;
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

    println!("Paint window, dirty rect: {dirty_rect:?}");
    wgpu_surface.configure_if_needed(window.handle.physical_size());

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
