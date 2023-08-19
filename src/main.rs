use rendering::{
    egui_pass::EguiRenderPass, opaque_pass::OpaqueRenderPass, renderer::Renderer,
    shadow_pass::ShadowRenderPass,
};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod rendering;

fn main() {
    puffin_egui::puffin::set_scopes_on(true);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Space game")
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(window);
    renderer.add_pass(ShadowRenderPass::new(&renderer));
    renderer.add_pass(OpaqueRenderPass::new());
    renderer.add_pass(EguiRenderPass::new(&renderer));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window.id() => {
                // let egui_pass = renderer.get_pass_mut::<EguiRenderPass>();
                // let response = egui_pass.state.on_event(&egui_pass.context, event);

                // if !response.consumed {
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
                    WindowEvent::Resized(_physical_size) => {
                        // painter.on_window_resized(physical_size.width, physical_size.height)
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size: _, ..
                    } => {
                        // painter.on_window_resized(new_inner_size.width, new_inner_size.height)
                    }
                    _ => {}
                }
                // }
            }
            Event::RedrawRequested(window_id) if window_id == renderer.window.id() => {
                puffin_egui::puffin::profile_function!();
                puffin_egui::puffin::GlobalProfiler::lock().new_frame();

                renderer.prepare();

                renderer.render();

                renderer.cleanup();
            }
            Event::MainEventsCleared => {
                renderer.window.request_redraw();
            }
            _ => {}
        }
    });
}
