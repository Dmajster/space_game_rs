use egui::Window;

use crate::app::{Res, ResMut};
use crate::editor::Editor;
use crate::Scene;

pub fn update(context: Res<egui::Context>, scene: ResMut<Scene>, editor: ResMut<Editor>) {
    let context = context.get();
    let mut editor = editor.get_mut();
    let mut scene = scene.get_mut();

    Window::new("Hierarchy").show(&context, |ui| {
        if ui.button("add scene object").clicked() {
            scene.add();
        }

        ui.separator();

        for scene_object in &scene.scene_object_hierarchy.nodes {
            if ui.button(format!("{}", scene_object.name)).clicked() {
                editor.selected_scene_object_id = scene_object.id;
            }
        }
    });
}
