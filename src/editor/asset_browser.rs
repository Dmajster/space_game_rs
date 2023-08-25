use egui::{ScrollArea, Window};
use native_dialog::FileDialog;

use crate::editor::Editor;
use crate::game::Game;
use crate::app::ResMut;

pub fn update(game: ResMut<Game>, egui: ResMut<Egui>, editor: ResMut<Editor>) {
    // Window::new("Asset browser").show(&egui.context, |ui| {
    //     let column_count = 5;

    //     if ui.button("add model").clicked() || editor.file_browser_open {
    //         let path = FileDialog::new()
    //             .add_filter("GLTF Model", &["gltf"])
    //             .show_open_single_file()
    //             .unwrap();

    //         if let Some(path) = path {
    //             println!("loading model from path: {:?}", &path);
    //             importing::gltf::load(&path, &mut app.asset_server);
    //         }
    //     };

    //     ui.separator();

    //     ScrollArea::vertical().show(ui, |scroll_area| {
    //         scroll_area.columns(column_count, |columns| {
    //             for mut i in 0..app.asset_server.get_meshes().len() {
    //                 let wrapped_index = i % column_count;

    //                 columns[wrapped_index].group(|ui| {
    //                     if let Some(mesh) = app.asset_server.get_mesh_at_index(i) {
    //                         ui.label(&mesh.name);
    //                         ui.label(format!("vertices: {}", mesh.positions.len()));
    //                         ui.label(format!("indices: {}", mesh.indices.len()));

    //                         if ui.button("delete").clicked() {
    //                             app.asset_server.remove_mesh_at_index(i);
    //                             i -= 1;
    //                         }
    //                     }
    //                 });
    //             }
    //         })
    //     });
    // });
}
