use glam::Mat4;
use serde::{Deserialize, Serialize};

use super::transform::TransformComponent;

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov_degrees: f32,
    pub aspect_w: f32,
    pub aspect_h: f32,
    pub z_near: f32,
}

impl CameraComponent {
    //TODO: cache this
    pub fn build_view_projection_matrix(&self, transform_component: &TransformComponent) -> Mat4 {
        let projection = Mat4::perspective_infinite_reverse_rh(
            self.fov_degrees.to_radians(),
            self.aspect_w / self.aspect_h,
            self.z_near,
        );

        projection * transform_component.build_transform_matrix().inverse()
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov_degrees: 90.0,
            aspect_w: 16.0,
            aspect_h: 9.0,
            z_near: 0.1,
        }
    }
}

pub fn draw(ui: &mut egui::Ui) {
    ui.heading("Camera");
    ui.add_space(8.0);

    ui.separator();
}
