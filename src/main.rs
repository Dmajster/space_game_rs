use app::{App, Passes};
use glam::{Mat4, Quat, Vec2, Vec3};
use rendering::{MeshDescriptor, MeshId};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod asset_server;
mod egui_pass;
mod importing;
mod opaque_pass;
mod rendering;
mod shadow_pass;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id(u64);

impl Id {
    pub fn new() -> Self {
        Id(fastrand::u64(..))
    }

    pub fn from_u64(val: u64) -> Self {
        Id(val)
    }

    pub const EMPTY: Id = Id(u64::MAX);
}

pub type SceneObjectId = Id;

#[derive(Debug, Default)]
pub struct Scene {
    pub scene_object_hierarchy: SceneObjectHierarchy,
}

impl Scene {
    pub fn add(&mut self) -> &mut SceneObject {
        self.scene_object_hierarchy.scene_objects.push(SceneObject {
            id: SceneObjectId::new(),
            parent: None,
            transform: Mat4::IDENTITY,
            mesh_id: MeshId::EMPTY,
        });

        self.scene_object_hierarchy
            .scene_objects
            .last_mut()
            .unwrap()
    }

    // pub fn parent(
    //     &mut self,
    //     node: Handle<SceneObjectHierarchyNode>,
    //     parent: Handle<SceneObjectHierarchyNode>,
    // ) {
    //     self.scene_object_hierarchy
    //         .nodes
    //         .get_mut(&node)
    //         .unwrap()
    //         .parent_handle = parent;
    // }
}

#[derive(Debug, Default)]
pub struct SceneObject {
    id: SceneObjectId,
    pub parent: Option<SceneObjectId>,
    pub transform: Mat4,
    pub mesh_id: MeshId,
}

#[derive(Debug, Default)]
pub struct SceneObjectHierarchy {
    pub scene_objects: Vec<SceneObject>,
}

fn main() {
    puffin_egui::puffin::set_scopes_on(true);

    let event_loop = EventLoop::new();

    let mut app = App::new(&event_loop);
    app.passes = Some(Rc::new(RefCell::new(Passes::new(&mut app))));

    let scene_object = app.scene.add();
    scene_object.transform = Mat4::from_scale_rotation_translation(
        Vec3::new(5.0, 5.0, 5.0),
        Quat::IDENTITY,
        Vec3::new(-2.5, 0.0, -2.5),
    );
    // scene_object.mesh_id = mesh.id();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.renderer.window.id() => {
                let response = {
                    let mut passes = app.passes.as_ref().unwrap().borrow_mut();
                    let context = passes.egui_pass.context.clone();

                    passes.egui_pass.state.on_event(&context, event)
                };

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
                        } => {
                            app.close();
                            *control_flow = ControlFlow::Exit
                        }
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
