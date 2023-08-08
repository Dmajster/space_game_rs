use rendering::{
    egui_pass::EguiRenderPass, opaque_pass::OpaqueRenderPass, renderer::Renderer, RenderPass,
};

use std::iter;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod rendering;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Space game")
        .with_inner_size(LogicalSize::new(1920.0, 1080.0))
        .build(&event_loop)
        .unwrap();

    let renderer = Renderer::new(window);

    let mut egui_pass = EguiRenderPass::new(&renderer);
    let mut opaque_pass = OpaqueRenderPass::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window.id() => {
                // if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        // painter.on_window_resized(physical_size.width, physical_size.height)
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // painter.on_window_resized(new_inner_size.width, new_inner_size.height)
                    }
                    _ => {}
                }
                // }
            }
            Event::RedrawRequested(window_id) if window_id == renderer.window.id() => {
                opaque_pass.prepare(&renderer);
                egui_pass.prepare(&renderer);

                let output = renderer.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    renderer
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                opaque_pass.render(&renderer, &mut encoder, &view);
                egui_pass.render(&renderer, &mut encoder, &view);

                renderer.queue.submit(iter::once(encoder.finish()));
                output.present();

                opaque_pass.cleanup(&renderer);
                egui_pass.cleanup(&renderer);
            }
            Event::MainEventsCleared => {
                renderer.window.request_redraw();
            }
            _ => {}
        }
    });
}
