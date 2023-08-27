use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LightComponent {}

pub fn draw(ui: &mut egui::Ui) {
    ui.heading("Light");
    ui.add_space(8.0);

    ui.separator();
}
