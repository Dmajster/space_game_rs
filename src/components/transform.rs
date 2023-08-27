use egui::*;
use glam::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

pub fn draw(ui: &mut egui::Ui, transform: &mut TransformComponent) {
    ui.heading("Transform");
    ui.add_space(8.0);

    ui.columns(4, |columns| {
        columns[0].label("Position");
        columns[1].add(
            DragValue::new(&mut transform.position.x)
                .speed(0.25)
                .suffix("m"),
        );
        columns[2].add(
            DragValue::new(&mut transform.position.y)
                .speed(0.25)
                .suffix("m"),
        );
        columns[3].add(
            DragValue::new(&mut transform.position.z)
                .speed(0.25)
                .suffix("m"),
        );
    });
    ui.columns(4, |columns| {
        columns[0].label("Rotation");
        columns[1].add(
            DragValue::new(&mut transform.rotation.x)
                .speed(1.0)
                .suffix("°"),
        );
        columns[2].add(
            DragValue::new(&mut transform.rotation.y)
                .speed(1.0)
                .suffix("°"),
        );
        columns[3].add(
            DragValue::new(&mut transform.rotation.z)
                .speed(1.0)
                .suffix("°"),
        );
    });
    ui.columns(4, |columns| {
        columns[0].label("Scale");
        columns[1].add(
            DragValue::new(&mut transform.scale.x)
                .speed(0.25)
                .suffix("x"),
        );
        columns[2].add(
            DragValue::new(&mut transform.scale.y)
                .speed(0.25)
                .suffix("x"),
        );
        columns[3].add(
            DragValue::new(&mut transform.scale.z)
                .speed(0.25)
                .suffix("x"),
        );
    });

    ui.separator();
}

impl TransformComponent {
    pub fn build_transform_matrix(&self) -> Mat4 {
        let cr = (self.rotation.x.to_radians() * 0.5).cos();
        let sr = (self.rotation.x.to_radians() * 0.5).sin();
        let cp = (self.rotation.y.to_radians() * 0.5).cos();
        let sp = (self.rotation.y.to_radians() * 0.5).sin();
        let cy = (self.rotation.z.to_radians() * 0.5).cos();
        let sy = (self.rotation.z.to_radians() * 0.5).sin();

        let rotation = Quat::from_xyzw(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        );

        Mat4::from_scale_rotation_translation(self.scale, rotation, self.position)
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}
