#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(tuple_trait)]

use app::App;
use asset_server::AssetServer;
use editor::Editor;
use game::Game;
use glam::{Mat4, Quat, Vec3};
use rendering::{MeshId, Renderer, RenderingRecorder};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::Path};
use ui::Egui;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod asset_server;
mod editor;
mod game;
mod importing;
mod rendering;
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

    let window = winit::window::WindowBuilder::new()
        .with_title("Space game")
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window);
    let egui = Egui::new(&window, &renderer);
    let editor = Editor::new();
    let asset_server = AssetServer::read_from_file_or_new(&asset_server::DEFAULT_PATH);
    let scene = Scene::read_from_file_or_new(&crate::DEFAULT_SCENE_PATH);
    let game = Game::new(&event_loop, &mut renderer, &window);

    let mut app = App::default();
    app.add_resource(game);
    app.add_resource(window);
    app.add_resource(renderer);
    app.add_resource(egui);
    app.add_resource(editor);
    app.add_resource(asset_server);
    app.add_resource(scene);
    app.add_system(game::update);
    app.add_system(ui::update);
    app.add_system(editor::update);
    app.add_system(editor::asset_browser::update);
    app.add_resource::<Option<RenderingRecorder>>(None);
    app.add_system(rendering::record);
    app.add_system(game::shadow_pass::render);
    app.add_system(game::opaque_pass::render);
    app.add_system(rendering::present);

    event_loop.run(move |event, _, control_flow| {
        let window = app.get_resource::<winit::window::Window>().unwrap();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.get().id() => {
                let egui = app.get_resource_mut::<Egui>().unwrap();
                let response = egui.get_mut().handle_event(&event);

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
                            // let event_queue = test_app
                            //     .get_resource_mut::<EventQueue<EventExitGame>>()
                            //     .unwrap();

                            // event_queue.get_mut().push_event(EventExitGame {});

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
            Event::MainEventsCleared => {
                puffin_egui::puffin::profile_function!();
                puffin_egui::puffin::GlobalProfiler::lock().new_frame();

                app.execute();

                // shadow_pass::render(&app, &mut encoder);
                // opaque_pass::render(&app, &mut encoder, &view);
                // ui::render(&mut app, &mut egui, &mut encoder, &view);

                // ui::post_render(&mut egui);
            }
            _ => {}
        }
    });
}
