use egui::ComboBox;
use serde::{Serialize, Deserialize};

use crate::asset_server::{AssetId, AssetServer};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MeshComponent {
    pub mesh_id: AssetId,
}

pub fn draw(
    ui: &mut egui::Ui,
    mesh_component: &mut MeshComponent,
    asset_server: &mut AssetServer,
) {
    ui.heading("Mesh");
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].label("Mesh ID:");

        ComboBox::from_label("")
            .selected_text(format!("{}", &mut mesh_component.mesh_id))
            .show_ui(&mut columns[1], |ui| {
                ui.selectable_value(&mut mesh_component.mesh_id, AssetId::EMPTY, "empty");

                for mesh in asset_server.meshes.iter() {
                    ui.selectable_value(
                        &mut mesh_component.mesh_id,
                        mesh.id(),
                        mesh.metadata.name.as_ref().unwrap(),
                    );
                }
            });
    });

    ui.separator();
}
