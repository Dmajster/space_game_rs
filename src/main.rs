use app::{App, Passes};
use glam::{Mat4, Quat, Vec3};
use rendering::MeshId;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, fmt, fs, path::Path, rc::Rc};

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod asset_server;
mod egui;
mod importing;
mod opaque_pass;
mod rendering;
mod shadow_pass;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id(u64);

pub const DEFAULT_SCENE_PATH: &'static str = "./scene.data";

impl Id {
    pub fn new() -> Self {
        Id(fastrand::u64(..))
    }

    pub fn from_u64(val: u64) -> Self {
        Id(val)
    }

    pub const EMPTY: Id = Id(u64::MAX);
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Id::EMPTY {
            f.write_str("empty")
        } else {
            f.write_str(&self.0.to_string())
        }
    }
}

pub type SceneObjectId = Id;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scene {
    pub scene_object_hierarchy: SceneObjectHierarchy,
}

impl Scene {
    pub fn read_from_file_or_new<P>(path: &P) -> Self
    where
        P: AsRef<Path>,
    {
        if let Ok(bytes) = fs::read(path) {
            let decompressed_bytes = lz4_flex::decompress_size_prepended(&bytes).unwrap();

            bincode::deserialize::<Self>(&decompressed_bytes).unwrap()
        } else {
            Default::default()
        }
    }

    pub fn write_to_file<P>(&self, path: &P)
    where
        P: AsRef<Path>,
    {
        let bytes = bincode::serialize::<Self>(&self).unwrap();

        let compressed_bytes = lz4_flex::compress_prepend_size(&bytes);

        fs::write(path, compressed_bytes).unwrap();
    }

    pub fn add(&mut self) -> &mut SceneObject {
        self.scene_object_hierarchy.scene_objects.push(SceneObject {
            id: SceneObjectId::new(),
            parent: None,
            mesh_id: MeshId::EMPTY,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        });

        self.scene_object_hierarchy
            .scene_objects
            .last_mut()
            .unwrap()
    }

    pub fn get_mut(&mut self, scene_object_id: SceneObjectId) -> &mut SceneObject {
        self.scene_object_hierarchy
            .scene_objects
            .iter_mut()
            .find(|scene_object| scene_object.id == scene_object_id)
            .unwrap()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SceneObject {
    id: SceneObjectId,
    pub parent: Option<SceneObjectId>,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub mesh_id: MeshId,
}

impl SceneObject {
    pub fn calculate_transform(&self) -> Mat4 {
        let cr = (self.rotation.x.to_radians() * 0.5).cos();
        let sr = (self.rotation.x.to_radians() * 0.5).sin();
        let cp = (self.rotation.y.to_radians() * 0.5).cos();
        let sp = (self.rotation.y.to_radians() * 0.5).sin();
        let cy = (self.rotation.z.to_radians() * 0.5).cos();
        let sy = (self.rotation.z.to_radians() * 0.5).sin();

        let rotation = Quat::from_xyzw(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        );

        Mat4::from_scale_rotation_translation(self.scale, rotation, self.position)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SceneObjectHierarchy {
    pub scene_objects: Vec<SceneObject>,
}

fn main() {
    puffin_egui::puffin::set_scopes_on(true);

    let event_loop = EventLoop::new();

    let mut app = App::new(&event_loop);
    app.passes = Some(Rc::new(RefCell::new(Passes::new(&mut app))));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.renderer.window.id() => {
                let response = { app.egui_state.on_event(&app.egui_context, event) };

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
