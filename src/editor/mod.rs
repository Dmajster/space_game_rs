use egui::{CentralPanel, ComboBox, DragValue, Frame};
use egui_dock::{DockArea, NodeIndex, Style, Tree};

use crate::{
    app::{Res, ResMut},
    asset_server::AssetServer,
    rendering::MeshId,
    Scene, SceneObjectId,
};

pub mod asset_browser;
pub mod hierarchy;
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

pub fn update(
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
                        editor: editor.clone(),
                        scene: scene.clone(),
                        asset_server: asset_server.clone(),
                    },
                );
        });
}

struct TabViewer {
    editor: ResMut<Editor>,
    scene: ResMut<Scene>,
    asset_server: ResMut<AssetServer>,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}
