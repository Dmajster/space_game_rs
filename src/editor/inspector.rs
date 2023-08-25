use egui::{ComboBox, DragValue, Window};

use crate::editor::Editor;
use crate::game::Game;
use crate::rendering::MeshId;
use crate::app::ResMut;
use crate::ui::Egui;
use crate::SceneObjectId;

pub fn update(game: ResMut<Game>, egui: ResMut<Egui>, editor: ResMut<Editor>) {
    // Window::new("Inspector").show(&egui.context, |ui| {
    //     if editor.selected_scene_object_id == SceneObjectId::EMPTY {
    //         return;
    //     }

    //     let sobj = app.scene.get_mut(editor.selected_scene_object_id);

    //     ui.columns(2, |columns| {
    //         columns[0].label("Name");
    //         columns[1].text_edit_singleline(&mut sobj.name);
    //     });

    //     ui.separator();

    //     ui.heading("Transform");
    //     ui.add_space(8.0);

    //     ui.columns(4, |columns| {
    //         columns[0].label("Position");
    //         columns[1].add(DragValue::new(&mut sobj.position.x).speed(0.25).suffix("m"));
    //         columns[2].add(DragValue::new(&mut sobj.position.y).speed(0.25).suffix("m"));
    //         columns[3].add(DragValue::new(&mut sobj.position.z).speed(0.25).suffix("m"));
    //     });
    //     ui.columns(4, |columns| {
    //         columns[0].label("Rotation");
    //         columns[1].add(DragValue::new(&mut sobj.rotation.x).speed(1.0).suffix("°"));
    //         columns[2].add(DragValue::new(&mut sobj.rotation.y).speed(1.0).suffix("°"));
    //         columns[3].add(DragValue::new(&mut sobj.rotation.z).speed(1.0).suffix("°"));
    //     });
    //     ui.columns(4, |columns| {
    //         columns[0].label("Scale");
    //         columns[1].add(DragValue::new(&mut sobj.scale.x).speed(0.25).suffix("x"));
    //         columns[2].add(DragValue::new(&mut sobj.scale.y).speed(0.25).suffix("x"));
    //         columns[3].add(DragValue::new(&mut sobj.scale.z).speed(0.25).suffix("x"));
    //     });

    //     ui.separator();

    //     ui.heading("Mesh");
    //     ui.add_space(8.0);

    //     ui.columns(2, |columns| {
    //         columns[0].label("Mesh ID:");

    //         ComboBox::from_label("")
    //             .selected_text(format!("{}", &mut sobj.mesh_id))
    //             .show_ui(&mut columns[1], |ui| {
    //                 ui.selectable_value(&mut sobj.mesh_id, MeshId::EMPTY, "empty");

    //                 for mesh in app.asset_server.get_meshes() {
    //                     ui.selectable_value(&mut sobj.mesh_id, mesh.id(), &mesh.name);
    //                 }
    //             });
    //     });

    //     ui.separator();
    // });
}
