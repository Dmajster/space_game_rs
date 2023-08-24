use crate::{rendering::RenderPass, App};
use egui::ClippedPrimitive;
use egui_wgpu::renderer::ScreenDescriptor;
use wgpu::{CommandEncoder, TextureView};

pub struct EguiRenderPass {
    renderer: egui_wgpu::Renderer,
    clipped_primitives: Vec<ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
}

impl EguiRenderPass {
    pub fn new(app: &App) -> Self {
        Self {
            renderer: egui_wgpu::Renderer::new(
                &app.renderer.device,
                app.renderer.surface_format,
                None,
                1,
            ),
            clipped_primitives: vec![],
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [
                    app.renderer.window.inner_size().width,
                    app.renderer.window.inner_size().height,
                ],
                pixels_per_point: 1.0,
            },
        }
    }
}

impl RenderPass for EguiRenderPass {
    fn prepare(&mut self, app: &App) {
        self.clipped_primitives = app
            .egui_context
            .tessellate(app.egui_full_output.shapes.clone()); // creates triangles to paint

        for (id, image_delta) in &app.egui_full_output.textures_delta.set {
            self.renderer.update_texture(
                &app.renderer.device,
                &app.renderer.queue,
                *id,
                image_delta,
            );
        }
    }

    fn render(&mut self, app: &App, mut encoder: &mut CommandEncoder, view: &TextureView) {
        self.renderer.update_buffers(
            &app.renderer.device,
            &app.renderer.queue,
            &mut encoder,
            self.clipped_primitives.as_slice(),
            &self.screen_descriptor,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.renderer.render(
            &mut render_pass,
            self.clipped_primitives.as_slice(),
            &self.screen_descriptor,
        );
    }

    fn cleanup(&mut self, app: &App) {
        for id in &app.egui_full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
