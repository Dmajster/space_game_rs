use egui::Window;

use crate::editor::Editor;
use crate::game::Game;
use crate::app::ResMut;
use crate::ui::Egui;

pub fn update(game: ResMut<Game>, egui: ResMut<Egui>, editor: ResMut<Editor>) {
    // Window::new("Hierarchy").show(&egui.context, |ui| {
    //     if ui.button("add scene object").clicked() {
    //         app.scene.add();
    //     }

    //     ui.separator();

    //     for scene_object in &app.scene.scene_object_hierarchy.nodes {
    //         if ui.button(format!("{}", scene_object.name)).clicked() {
    //             editor.selected_scene_object_id = scene_object.id;
    //         }
    //     }
    // });
}
