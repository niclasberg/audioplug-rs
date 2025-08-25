use wgpu::{
    RenderPassColorAttachment, RenderPassDescriptor,
    wgt::{CommandEncoderDescriptor, TextureViewDescriptor},
};

use crate::{
    core::Rect,
    ui::{AppState, WindowId},
};

mod wgpu_surface;
pub use wgpu_surface::{GraphicsInitError, WGPUSurface};

pub fn paint_window(app_state: &mut AppState, window_id: WindowId, dirty_rect: Rect) {
    println!("Paint window, dirty rect: {dirty_rect:?}");

    let window = &mut app_state.windows[window_id];
    let wgpu_surface = &mut window.wgpu_surface;
    let surface_texture = wgpu_surface
        .surface
        .get_current_texture()
        .expect("Unable to get surface texture");
    let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
        format: Some(wgpu_surface.surface_format.add_srgb_suffix()),
        ..Default::default()
    });

    let mut encoder = wgpu_surface
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("AudioPlug command encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
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
        render_pass.set_pipeline(&wgpu_surface.blit_pipeline);
        render_pass.set_bind_group(0, &wgpu_surface.blit_bind_group, &[]);
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
