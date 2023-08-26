#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(tuple_trait)]

use app::App;
use asset_server::AssetServer;
use editor::Editor;
use game::Game;
use rendering::{Renderer, RenderingRecorder};
use scene::Scene;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

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
mod scene;
mod ui;
mod components;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

impl Default for Id {
    fn default() -> Self {
        Self::EMPTY
    }
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

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Id::EMPTY {
            f.write_str("empty")
        } else {
            f.debug_tuple("Id").field(&self.0).finish()
        }
    }
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
    let editor = Editor::new();
    let asset_server = AssetServer::read_from_file_or_new(&asset_server::DEFAULT_PATH);
    let scene = Scene::read_from_file_or_new(&scene::DEFAULT_SCENE_PATH);
    let game = Game::new(&mut renderer, &window);

    let mut app = App::default();

    // EGUI
    app.add_resource(egui::Context::default());
    app.add_resource(egui_winit::State::new(&window));
    app.add_resource(egui::FullOutput::default());
    app.add_resource(egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.surface_format,
        None,
        1,
    ));
    app.add_resource::<Vec<egui::ClippedPrimitive>>(vec![]);
    app.add_resource(egui_wgpu::renderer::ScreenDescriptor {
        size_in_pixels: [window.inner_size().width, window.inner_size().height],
        pixels_per_point: 1.0,
    });
    app.add_system(ui::update);
    //

    // DOCKING EDITOR
    // let tree = {
    //     let mut tree = Tree::new(vec!["hierarchy".to_string()]);
    //     let [hierarchy, inspector] =
    //         tree.split_right(NodeIndex::root(), 0.8, vec!["inspector".to_string()]);

    //     let [hierarchy, file_browser] = tree.split_below(
    //         hierarchy,
    //         0.6,
    //         vec!["file browser".to_string(), "profiler".to_string()],
    //     );

    //     let [hierarchy, scene] = tree.split_right(hierarchy, 0.25, vec!["scene".to_string()]);
    //     tree.split_right(scene, 0.5, vec!["game".to_string()]);
    //     tree
    // };
    // app.add_resource::<Tree<String>>(tree);
    // app.add_system(editor::update);
    //

    // RENDERER
    app.add_resource(renderer);
    app.add_system(rendering::update_scene_object_transforms);
    //

    // OLD EDITOR
    app.add_system(editor::scene_hierarchy::update);
    app.add_raw_system(editor::inspector::update);
    app.add_system(editor::asset_browser::update);
    //

    app.add_resource(game);
    app.add_system(game::update);

    app.add_resource(window);
    app.add_resource(editor);
    app.add_resource(asset_server);
    app.add_resource(scene);

    app.add_resource::<Option<RenderingRecorder>>(None);
    app.add_system(rendering::record);
    app.add_system(game::shadow_pass::render);
    app.add_system(game::opaque_pass::render);
    app.add_system(ui::render);
    app.add_system(rendering::present);
    app.add_system(ui::post_render);

    event_loop.run(move |event, _, control_flow| {
        let window = app.get_resource::<winit::window::Window>().unwrap();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.get().id() => {
                let context = app.get_resource::<egui::Context>().unwrap();
                let state = app.get_resource_mut::<egui_winit::State>().unwrap();
                let response = state.get_mut().on_event(&context.get(), event);

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
            }
            _ => {}
        }
    });
}
