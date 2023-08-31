use std::mem;

use egui::{ComboBox, DragValue};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightType {
    DirectionalLight = 0,
    PointLight = 1,
    SpotLight = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LightComponent {
    pub ty: LightType,
    pub color: Vec3,
    pub luminous_intensity: f32,
    pub falloff_radius: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            luminous_intensity: 5.0,
            inner_angle: 0.0,
            color: Vec3::ONE,
            outer_angle: 0.0,
            falloff_radius: 0.0,
            ty: LightType::DirectionalLight,
        }
    }
}

pub fn draw(ui: &mut egui::Ui, light_component: &mut LightComponent) {
    ui.heading("Light");
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].label("type:");
        ComboBox::from_label("")
            .selected_text(format!("{:?}", light_component.ty))
            .show_ui(&mut columns[1], |ui| {
                for i in 0i32..mem::variant_count::<LightType>() as i32 {
                    let light_type = unsafe { mem::transmute(i) };

                    ui.selectable_value(
                        &mut light_component.ty,
                        light_type,
                        format!("{:?}", light_type),
                    );
                }
            });
    });
    ui.columns(2, |columns| {
        columns[0].label("color:");
        columns[1].color_edit_button_rgb(light_component.color.as_mut().into());
    });

    ui.columns(2, |columns| {
        columns[0].label("luminous intensity:");
        columns[1].add(
            DragValue::new(&mut light_component.luminous_intensity)
                .speed(1.0)
                .suffix(" (cd)"),
        );
    });

    if light_component.ty != LightType::DirectionalLight {
        ui.columns(2, |columns| {
            columns[0].label("falloff radius:");
            columns[1].add(
                DragValue::new(&mut light_component.falloff_radius)
                    .clamp_range(0.0..=1000.0)
                    .speed(1.0)
                    .suffix("m"),
            );
        });
    }

    if light_component.ty == LightType::SpotLight {
        ui.columns(2, |columns| {
            columns[0].label("outer angle:");
            columns[1].add(
                DragValue::new(&mut light_component.outer_angle)
                    .clamp_range(0.0..=180.0)
                    .speed(1.0)
                    .suffix("°"),
            );
        });

        ui.columns(2, |columns| {
            columns[0].label("inner angle:");
            columns[1].add(
                DragValue::new(&mut light_component.inner_angle)
                    .clamp_range(0.0..=light_component.outer_angle)
                    .speed(1.0)
                    .suffix("°"),
            );
        });
    }

    ui.separator();
}
