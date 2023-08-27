use egui::*;
use native_dialog::FileDialog;

use crate::app::{Res, ResMut};
use crate::asset_server::{self, AssetServer};
use crate::editor::Editor;
use crate::importing;

pub fn update(editor: Res<Editor>, context: Res<egui::Context>, asset_server: ResMut<AssetServer>) {
    let context = context.get();
    let editor = editor.get();
    let mut asset_server = asset_server.get_mut();

    Window::new("Asset browser")
        .min_width(512.0)
        .show(&context, |ui| {
            let column_count = 5;

            if ui.button("add model").clicked() || editor.file_browser_open {
                let path = FileDialog::new()
                    .add_filter("GLTF Model", &["gltf"])
                    .show_open_single_file()
                    .unwrap();

                if let Some(path) = path {
                    println!("loading model from path: {:?}", &path);
                    importing::gltf::load(&path, &mut asset_server);
                }
            };

            if ui.button("save assets").clicked() {
                asset_server.write_to_file(&asset_server::DEFAULT_PATH);
            }

            ui.separator();

            ScrollArea::vertical().show(ui, |scroll_area| {
                let mut models = asset_server.models_mut();
                
                scroll_area.columns(column_count, |columns| {
                    for mut i in 0..models.len() as isize {
                        let wrapped_index = i as usize % column_count;

                        columns[wrapped_index].group(|ui| {
                            if let Some(mesh) = models.get_at_index(i as usize) {
                                ui.label(mesh.metadata.name.as_ref().unwrap());

                                if ui.button("delete").clicked() {
                                    models.remove_at_index(i as usize);

                                    i -= 1;
                                }
                            }
                        });
                    }
                })
            });
        });
}
