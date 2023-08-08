use super::{renderer::Renderer, RenderPass};
use egui::{ClippedPrimitive, FullOutput, Pos2, RawInput, Rect};
use egui_wgpu::renderer::ScreenDescriptor;
use wgpu::{CommandEncoder, TextureView};

pub struct EguiRenderPass {
    pub context: egui::Context,
    renderer: egui_wgpu::Renderer,
    raw_input: RawInput,
    clipped_primitives: Vec<ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
    full_output: FullOutput,
}

impl EguiRenderPass {
    pub fn new(renderer: &Renderer) -> Self {
        Self {
            context: egui::Context::default(),
            renderer: egui_wgpu::Renderer::new(&renderer.device, renderer.surface_format, None, 1),
            clipped_primitives: vec![],
            raw_input: RawInput {
                screen_rect: Some(Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 {
                        x: renderer.window.inner_size().width as f32,
                        y: renderer.window.inner_size().height as f32,
                    },
                }),
                pixels_per_point: Some(renderer.window.scale_factor() as f32),
                max_texture_side: Default::default(),
                ..Default::default()
            },
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [
                    renderer.window.inner_size().width,
                    renderer.window.inner_size().height,
                ],
                pixels_per_point: 1.0,
            },
            full_output: FullOutput::default(),
        }
    }
}

impl RenderPass for EguiRenderPass {
    fn prepare(&mut self, renderer: &Renderer) {
        self.full_output = self.context.run(self.raw_input.clone(), |context| {
            egui::SidePanel::new(egui::panel::Side::Left, "Debug")
                .show(context, |ui| ui.label("test"));
        });

        self.clipped_primitives = self.context.tessellate(self.full_output.shapes.clone()); // creates triangles to paint

        for (id, image_delta) in &self.full_output.textures_delta.set {
            self.renderer
                .update_texture(&renderer.device, &renderer.queue, *id, image_delta);
        }
    }

    fn render(
        &mut self,
        renderer: &Renderer,
        mut encoder: &mut CommandEncoder,
        view: &TextureView,
    ) {
        self.renderer.update_buffers(
            &renderer.device,
            &renderer.queue,
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

    fn cleanup(&mut self, _renderer: &Renderer) {
        for id in &self.full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
