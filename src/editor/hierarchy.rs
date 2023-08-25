use egui::Window;

use crate::app::App;
use crate::editor::Editor;
use crate::ui::Egui;

pub fn update(app: &mut App, egui: &mut Egui, editor: &mut Editor) {
    Window::new("Hierarchy").show(&egui.context, |ui| {
        if ui.button("add scene object").clicked() {
            app.scene.add();
        }

        ui.separator();

        for scene_object in &app.scene.scene_object_hierarchy.nodes {
            if ui.button(format!("{}", scene_object.name)).clicked() {
                editor.selected_scene_object_id = scene_object.id;
            }
        }
    });
}
