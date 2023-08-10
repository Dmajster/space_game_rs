use super::{renderer::Renderer, RenderPass};
use egui::{ClippedPrimitive, FullOutput};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::State;
use wgpu::{CommandEncoder, TextureView};

pub struct EguiRenderPass {
    pub context: egui::Context,
    pub state: State,

    renderer: egui_wgpu::Renderer,
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
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [
                    renderer.window.inner_size().width,
                    renderer.window.inner_size().height,
                ],
                pixels_per_point: 1.0,
            },
            full_output: FullOutput::default(),
            state: egui_winit::State::new(&renderer.window),
        }
    }
}

impl RenderPass for EguiRenderPass {
    fn prepare(&mut self, renderer: &Renderer) {
        let raw_input = self.state.take_egui_input(&renderer.window);

        self.full_output = self.context.run(raw_input, |context| {
            egui::SidePanel::new(egui::panel::Side::Left, "Profiler")
                .show(context, |ui| puffin_egui::profiler_ui(ui));
        });

        self.state.handle_platform_output(
            &renderer.window,
            &self.context,
            self.full_output.platform_output.clone(),
        );

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
