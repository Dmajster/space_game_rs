use egui::{CentralPanel, Frame};
use egui_dock::{DockArea, Style, Tree};

use crate::{
    app::{Res, ResMut},
    asset_server::AssetServer,
    scene::SceneObjectId,
    Scene,
};

pub mod asset_browser;
pub mod scene_hierarchy;
pub mod inspector;

pub struct Editor {
    selected_scene_object_id: SceneObjectId,
    file_browser_open: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            selected_scene_object_id: SceneObjectId::EMPTY,
            file_browser_open: false,
        }
    }
}

pub fn _update(
    context: Res<egui::Context>,
    editor: ResMut<Editor>,
    scene: ResMut<Scene>,
    asset_server: ResMut<AssetServer>,
    tree: ResMut<Tree<String>>,
) {
    let context = context.get();

    CentralPanel::default()
        .frame(Frame::central_panel(&context.style()).inner_margin(0.))
        .show(&context, |_| {
            DockArea::new(&mut tree.get_mut())
                .style(Style::from_egui(context.style().as_ref()))
                .show(
                    &context,
                    &mut TabViewer {
                        _editor: editor.clone(),
                        _scene: scene.clone(),
                        _asset_server: asset_server.clone(),
                    },
                );
        });
}

struct TabViewer {
    _editor: ResMut<Editor>,
    _scene: ResMut<Scene>,
    _asset_server: ResMut<AssetServer>,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn ui(&mut self, _ui: &mut egui::Ui, _tab: &mut Self::Tab) {}

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}
