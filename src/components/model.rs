use egui::ComboBox;
use serde::{Deserialize, Serialize};

use crate::{
    asset_server::{asset_id::AssetId, AssetServer},
    rendering::model::Model,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ModelComponent {
    pub model_id: AssetId<Model>,
}

pub fn draw(
    ui: &mut egui::Ui,
    model_component: &mut ModelComponent,
    asset_server: &mut AssetServer,
) {
    ui.heading("Model");
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].label("Model ID:");

        ComboBox::from_label("")
            .selected_text(format!("{}", model_component.model_id))
            .show_ui(&mut columns[1], |ui| {
                ui.selectable_value(
                    &mut model_component.model_id,
                    AssetId::<Model>::EMPTY,
                    "empty",
                );

                for model in asset_server.models().iter() {
                    ui.selectable_value(
                        &mut model_component.model_id,
                        model.id(),
                        model.metadata.name.as_ref().unwrap(),
                    );
                }
            });
    });

    ui.separator();
}
