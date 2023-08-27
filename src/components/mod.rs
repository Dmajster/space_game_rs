use egui::ComboBox;
use serde::{Deserialize, Serialize};

use crate::asset_server::{AssetId, AssetServer};
use std::fmt::Debug;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MeshComponent {
    pub mesh_id: AssetId,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LightComponent {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CameraComponent {}

pub fn draw_mesh_component(
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

pub fn draw_light_component(
    ui: &mut egui::Ui,
) {
    ui.heading("Light");
    ui.add_space(8.0);

    ui.separator();
}

pub fn draw_camera_component(
    ui: &mut egui::Ui,
) {
    ui.heading("Camera");
    ui.add_space(8.0);

    ui.separator();
}