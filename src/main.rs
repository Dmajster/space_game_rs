use app::{App, Passes};
use glam::{Mat4, Quat, Vec2, Vec3};
use rendering::RenderSceneObject;
use std::{cell::RefCell, rc::Rc};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod importing;
mod rendering;
mod egui_pass;
mod opaque_pass;
mod shadow_pass;

pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

pub struct Sun {
    pub inverse_direction: Vec3,
    pub projection: Mat4,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunUniform {
    mvp: Mat4,
}

impl SunUniform {
    pub fn update(&mut self, sun: &Sun) {
        let view = Mat4::look_at_rh(sun.inverse_direction, Vec3::ZERO, Vec3::Y);
        self.mvp = sun.projection * view;
    }
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

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: Mat4,
}

impl CameraUniform {
    fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix();
    }
}

fn main() {
    puffin_egui::puffin::set_scopes_on(true);

    let event_loop = EventLoop::new();

    let mut app = App::new(&event_loop);

    app.passes = Some(Rc::new(RefCell::new(Passes::new(&mut app))));

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

    let mesh_handle = app.renderer.add_mesh(mesh);

    app.renderer.scene_objects.push(RenderSceneObject {
        transform: Mat4::from_scale_rotation_translation(
            Vec3::new(5.0, 5.0, 5.0),
            Quat::IDENTITY,
            Vec3::new(-2.5, 0.0, -2.5),
        ),
        mesh_handle,
    });
    app.renderer.scene_objects.push(RenderSceneObject {
        transform: Mat4::from_scale_rotation_translation(
            Vec3::new(1.0, 1.0, 1.0),
            Quat::IDENTITY,
            Vec3::new(-0.5, 1.0, -0.5),
        ),
        mesh_handle,
    });

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.renderer.window.id() => {
                let mut passes = app.passes.as_ref().unwrap().borrow_mut();
                let context = passes.egui_pass.context.clone();

                let response = passes.egui_pass.state.on_event(&context, event);

                if !response.consumed {
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
                }
            }
            Event::RedrawRequested(window_id) if window_id == app.renderer.window.id() => {
                puffin_egui::puffin::profile_function!();
                puffin_egui::puffin::GlobalProfiler::lock().new_frame();

                app.pre_update();

                app.update();

                app.post_update();
            }
            Event::MainEventsCleared => {
                app.renderer.window.request_redraw();
            }
            _ => {}
        }
    });
}
