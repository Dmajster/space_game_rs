use egui::{DragValue, Ui, Window};

use crate::app::{Res, ResMut};
use crate::asset_server::AssetServer;
use crate::components::{
    draw_camera_component, draw_light_component, draw_mesh_component, CameraComponent,
    LightComponent, MeshComponent,
};
use crate::editor::Editor;
use crate::scene::SceneObjectId;
use crate::Scene;

pub fn update(
    context: Res<egui::Context>,
    editor: Res<Editor>,
    scene: ResMut<Scene>,
    asset_server: ResMut<AssetServer>,
) {
    let context = context.get();
    let editor = editor.get();
    let mut scene = scene.get_mut();
    let mut asset_server = asset_server.get_mut();

    Window::new("Inspector")
        .min_width(512.0)
        .show(&context, |ui| {
            if editor.selected_scene_object_id == SceneObjectId::EMPTY {
                return;
            }

            let sobj = scene.get_mut(editor.selected_scene_object_id).unwrap();

            ui.columns(2, |columns| {
                columns[0].label("Name");
                columns[1].text_edit_singleline(&mut sobj.name);
            });

            ui.separator();

            ui.heading("Transform");
            ui.add_space(8.0);

            ui.columns(4, |columns| {
                columns[0].label("Position");
                columns[1].add(DragValue::new(&mut sobj.position.x).speed(0.25).suffix("m"));
                columns[2].add(DragValue::new(&mut sobj.position.y).speed(0.25).suffix("m"));
                columns[3].add(DragValue::new(&mut sobj.position.z).speed(0.25).suffix("m"));
            });
            ui.columns(4, |columns| {
                columns[0].label("Rotation");
                columns[1].add(DragValue::new(&mut sobj.rotation.x).speed(1.0).suffix("°"));
                columns[2].add(DragValue::new(&mut sobj.rotation.y).speed(1.0).suffix("°"));
                columns[3].add(DragValue::new(&mut sobj.rotation.z).speed(1.0).suffix("°"));
            });
            ui.columns(4, |columns| {
                columns[0].label("Scale");
                columns[1].add(DragValue::new(&mut sobj.scale.x).speed(0.25).suffix("x"));
                columns[2].add(DragValue::new(&mut sobj.scale.y).speed(0.25).suffix("x"));
                columns[3].add(DragValue::new(&mut sobj.scale.z).speed(0.25).suffix("x"));
            });

            ui.separator();

            if let Some(mesh_component) = &mut sobj.mesh_component {
                draw_mesh_component(ui, mesh_component, &mut asset_server);
            }

            if let Some(light_component) = &mut sobj.light_component {
                draw_light_component(ui);
            }

            if let Some(camera_component) = &mut sobj.camera_component {
                draw_camera_component(ui);
            }
        })
        .unwrap()
        .response
        .context_menu(|ui| {
            if editor.selected_scene_object_id != SceneObjectId::EMPTY {
                ui_tree_context_menu(ui, &editor, &mut scene);
            }
        });
}

fn ui_tree_context_menu(ui: &mut Ui, editor: &Editor, scene: &mut Scene) {
    let scene_object = scene.get_mut(editor.selected_scene_object_id).unwrap();

    if ui.button("add mesh").clicked() {
        scene_object.mesh_component = Some(MeshComponent::default());
        ui.close_menu();
    }
    if ui.button("add light").clicked() {
        scene_object.light_component = Some(LightComponent::default());
        ui.close_menu();
    }
    if ui.button("add camera").clicked() {
        scene_object.camera_component = Some(CameraComponent::default());
        ui.close_menu();
    }
}
