use egui::ComboBox;
use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    asset_server::{AssetId, AssetServer},
    components::Component,
    editor::inspector::EditorInspector,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MeshComponent {
    pub mesh_id: AssetId,
}

#[typetag::serde]
impl Component for MeshComponent {}

impl EditorInspector for MeshComponent {
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut App) {
        let asset_server = app.get_resource_mut::<AssetServer>().unwrap();
        let asset_server = asset_server.get_mut();

        ui.columns(2, |columns| {
            columns[0].label("Mesh ID:");

            ComboBox::from_label("")
                .selected_text(format!("{}", &mut self.mesh_id))
                .show_ui(&mut columns[1], |ui| {
                    ui.selectable_value(&mut self.mesh_id, AssetId::EMPTY, "empty");

                    for mesh in asset_server.meshes.iter() {
                        ui.selectable_value(
                            &mut self.mesh_id,
                            mesh.id(),
                            mesh.metadata.name.as_ref().unwrap(),
                        );
                    }
                });
        });
    }
}
