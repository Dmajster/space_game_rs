use crate::app::Res;
use crate::app::ResMut;
use crate::rendering::Renderer;
use crate::rendering::RenderingRecorder;
use winit::window::Window;

pub fn update(window: Res<Window>, state: ResMut<egui_winit::State>, context: Res<egui::Context>) {
    let mut state = state.get_mut();
    let window = window.get();
    let raw_input = state.take_egui_input(&window);

    context.get().begin_frame(raw_input);
}

pub fn render(
    window: ResMut<Window>,
    renderer: ResMut<Renderer>,
    context: Res<egui::Context>,
    egui_renderer: ResMut<egui_wgpu::Renderer>,
    full_output: ResMut<egui::FullOutput>,
    state: ResMut<egui_winit::State>,
    clipped_primitives: ResMut<Vec<egui::ClippedPrimitive>>,
    rendering_recorder: ResMut<Option<RenderingRecorder>>,
    screen_descriptor: Res<egui_wgpu::renderer::ScreenDescriptor>,
) {
    let window = window.get_mut();
    let renderer = renderer.get_mut();
    let mut rendering_recorder = rendering_recorder.get_mut();
    let rendering_recorder = rendering_recorder.as_mut().unwrap();
    let screen_descriptor = screen_descriptor.get();
    let mut egui_renderer = egui_renderer.get_mut();
    let mut state = state.get_mut();

    *full_output.get_mut() = context.get().end_frame();

    *clipped_primitives.get_mut() = context
        .get()
        .tessellate(full_output.get_mut().shapes.clone()); // creates triangles to paint

    for (id, image_delta) in &full_output.get_mut().textures_delta.set {
        egui_renderer.update_texture(&renderer.device, &renderer.queue, *id, image_delta);
    }

    state.handle_platform_output(
        &window,
        &context.get(),
        full_output.get_mut().platform_output.clone(),
    );

    egui_renderer.update_buffers(
        &renderer.device,
        &renderer.queue,
        &mut rendering_recorder.encoder,
        clipped_primitives.get_mut().as_slice(),
        &screen_descriptor,
    );

    let mut render_pass =
        rendering_recorder
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &rendering_recorder.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

    egui_renderer.render(
        &mut render_pass,
        clipped_primitives.get_mut().as_slice(),
        &screen_descriptor,
    );
}

pub fn post_render(full_output: Res<egui::FullOutput>, egui_renderer: ResMut<egui_wgpu::Renderer>) {
    let mut egui_renderer = egui_renderer.get_mut();

    for id in full_output.get().textures_delta.free.iter() {
        egui_renderer.free_texture(&id);
    }
}
