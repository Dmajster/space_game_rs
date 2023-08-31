use egui::DragValue;
use glam::Mat4;
use serde::{Deserialize, Serialize};

use super::transform::TransformComponent;

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov_degrees: f32,
    pub aspect_w: f32,
    pub aspect_h: f32,
    pub z_near: f32,

    pub aperture_f_stops: f32,
    pub shutter_speed_1_over_seconds: f32,
    pub sensitivity_iso: f32,
}

impl CameraComponent {
    //TODO: cache this
    pub fn calculate_view_projection_matrix(
        &self,
        transform_component: &TransformComponent,
    ) -> Mat4 {
        let projection = Mat4::perspective_infinite_reverse_rh(
            self.fov_degrees.to_radians(),
            self.aspect_w / self.aspect_h,
            self.z_near,
        );

        projection * transform_component.build_transform_matrix().inverse()
    }

    pub fn calculate_exposure(&self) -> f32 {
        let shutter_speed = 1.0 / self.shutter_speed_1_over_seconds;

        let ev100 =
            (self.aperture_f_stops.powf(2.0) / shutter_speed * 100.0 / self.sensitivity_iso).log2();

        let exposure = 1.0 / 2.0f32.powf(ev100) * 1.2;

        exposure
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov_degrees: 90.0,
            aspect_w: 16.0,
            aspect_h: 9.0,
            z_near: 0.1,
            aperture_f_stops: 2.8,
            shutter_speed_1_over_seconds: 2.0,
            sensitivity_iso: 1600.0,
        }
    }
}

pub fn draw(ui: &mut egui::Ui, camera: &mut CameraComponent) {
    ui.heading("Camera");
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].label("fov: ");
        columns[1].add(
            DragValue::new(&mut camera.fov_degrees)
                .speed(0.1)
                .suffix("°"),
        );
    });
    ui.columns(2, |columns| {
        columns[0].label("z_near: ");
        columns[1].add(DragValue::new(&mut camera.z_near).speed(0.1).suffix("m"));
    });
    ui.columns(2, |columns| {
        columns[0].label("aperture: ");
        columns[1].add(DragValue::new(&mut camera.aperture_f_stops).speed(1).suffix(" (f stops)"));
    });
    ui.columns(2, |columns| {
        columns[0].label("shutter speed: ");
        columns[1].add(DragValue::new(&mut camera.shutter_speed_1_over_seconds).speed(1).prefix("1/"));
    });
    ui.columns(2, |columns| {
        columns[0].label("sensitivity: ");
        columns[1].add(DragValue::new(&mut camera.sensitivity_iso).speed(10).suffix(" (iso)"));
    });

    ui.separator();
}
