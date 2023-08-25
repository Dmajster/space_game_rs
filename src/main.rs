use app::App;
use editor::Editor;
use glam::{Mat4, Quat, Vec3};
use rendering::MeshId;
use serde::{Deserialize, Serialize};
use std::{fmt, fs, iter, path::Path};
use ui::Egui;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod asset_server;
mod editor;
mod importing;
mod opaque_pass;
mod rendering;
mod shadow_pass;
mod ui;

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
    pub scene_object_hierarchy: SceneHierarchy,
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
        self.scene_object_hierarchy.nodes.push(SceneObject {
            name: String::from("Scene object"),
            id: SceneObjectId::new(),
            parent: None,
            mesh_id: MeshId::EMPTY,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        });

        self.scene_object_hierarchy.nodes.last_mut().unwrap()
    }

    pub fn get_mut(&mut self, scene_object_id: SceneObjectId) -> &mut SceneObject {
        self.scene_object_hierarchy
            .nodes
            .iter_mut()
            .find(|scene_object| scene_object.id == scene_object_id)
            .unwrap()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SceneObject {
    pub name: String,
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
pub struct SceneHierarchy {
    pub nodes: Vec<SceneObject>,
}

fn main() {
    puffin_egui::puffin::set_scopes_on(true);

    let event_loop = EventLoop::new();

    let mut app = App::new(&event_loop);

    let mut egui = Egui::new(&app.renderer);

    let mut editor = Editor::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.renderer.window.id() => {
                let response = egui.handle_event(&event);

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
                            app::close(&mut app);
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

                app::update(&mut app);

                ui::update(&mut app, &mut egui);
                editor::update(&mut app, &mut egui, &mut editor);
                editor::asset_browser::update(&mut app, &mut egui, &mut editor);
                editor::hierarchy::update(&mut app, &mut egui, &mut editor);
                editor::inspector::update(&mut app, &mut egui, &mut editor);

                let output = app.renderer.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    app.renderer
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                shadow_pass::render(&app, &mut encoder);
                opaque_pass::render(&app, &mut encoder, &view);
                ui::render(&mut app, &mut egui, &mut encoder, &view);

                app.renderer.queue.submit(iter::once(encoder.finish()));
                output.present();

                ui::post_render(&mut egui);
            }
            Event::MainEventsCleared => {
                app.renderer.window.request_redraw();
            }
            _ => {}
        }
    });
}
