use crate::rendering::Renderer;
use egui::ClippedPrimitive;
use egui::FullOutput;
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::EventResponse;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct Egui {
    pub context: egui::Context,
    state: egui_winit::State,
    full_output: egui::FullOutput,
    renderer: egui_wgpu::Renderer,
    clipped_primitives: Vec<ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
}

impl Egui {
    pub fn new(window: &Window, renderer: &Renderer) -> Self {
        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(&window);
        let egui_full_output = FullOutput::default();

        Self {
            context: egui_context,
            state: egui_state,
            full_output: egui_full_output,
            renderer: egui_wgpu::Renderer::new(&renderer.device, renderer.surface_format, None, 1),
            clipped_primitives: vec![],
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [window.inner_size().width, window.inner_size().height],
                pixels_per_point: 1.0,
            },
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.state.on_event(&self.context, event)
    }
}

pub fn update(window: &mut Window, egui: &mut Egui) {
    let raw_input = egui.state.take_egui_input(&window);

    egui.context.begin_frame(raw_input);
}

pub fn render(
    window: &mut Window,
    renderer: &mut Renderer,
    egui: &mut Egui,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    egui.full_output = egui.context.end_frame();

    egui.clipped_primitives = egui.context.tessellate(egui.full_output.shapes.clone()); // creates triangles to paint

    for (id, image_delta) in &egui.full_output.textures_delta.set {
        egui.renderer
            .update_texture(&renderer.device, &renderer.queue, *id, image_delta);
    }

    egui.state.handle_platform_output(
        &window,
        &egui.context,
        egui.full_output.platform_output.clone(),
    );

    egui.renderer.update_buffers(
        &renderer.device,
        &renderer.queue,
        encoder,
        egui.clipped_primitives.as_slice(),
        &egui.screen_descriptor,
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

    egui.renderer.render(
        &mut render_pass,
        egui.clipped_primitives.as_slice(),
        &egui.screen_descriptor,
    );
}

pub fn post_render(egui: &mut Egui) {
    for id in &egui.full_output.textures_delta.free {
        egui.renderer.free_texture(id);
    }
}
