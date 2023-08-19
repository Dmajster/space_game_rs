use glam::{Mat4, Vec2, Vec3};
use rendering::{
    opaque_pass::OpaqueRenderPass,
    renderer::{RenderMesh, RenderSceneObject, Renderer},
    shadow_pass::ShadowRenderPass,
};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod rendering;

pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct Sun {
    pub inverse_direction: Vec3,
    pub projection: Mat4,
}

pub struct Camera {
    pub transform: Mat4,
    pub projection: Mat4,
}

impl Camera {
    fn new(transform: Mat4, projection: Mat4) -> Self {
        Self {
            transform,
            projection,
        }
    }

    fn build_view_projection_matrix(&self) -> Mat4 {
        self.projection * self.transform
    }
}

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
    renderer.add_pass(OpaqueRenderPass::new(&renderer));
    // renderer.add_pass(EguiRenderPass::new(&renderer));

    let mesh = Mesh {
        positions: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        ],
        normals: vec![Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y],
        uvs: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ],
        indices: vec![0, 1, 2, 2, 3, 0],
    };

    let mesh_handle = renderer.add_mesh(mesh);

    renderer.scene_objects.push(RenderSceneObject {
        transform: Mat4::IDENTITY,
        mesh_handle,
    });

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
