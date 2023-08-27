use egui::{Ui, Window};

use crate::app::{Res, ResMut};
use crate::asset_server::AssetServer;
use crate::components::camera::CameraComponent;
use crate::components::light::LightComponent;
use crate::components::model::ModelComponent;
use crate::editor::Editor;
use crate::scene::SceneObjectId;
use crate::{components, Scene};

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
                columns[0].heading("Name");
                columns[1].text_edit_singleline(&mut sobj.name);
            });

            ui.separator();

            components::transform::draw(ui, &mut sobj.transform_component);

            if let Some(model_component) = &mut sobj.model_component {
                components::model::draw(ui, model_component, &mut asset_server);
            }

            if let Some(light_component) = &mut sobj.light_component {
                components::light::draw(ui);
            }

            if let Some(camera_component) = &mut sobj.camera_component {
                components::camera::draw(ui);
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

    if ui.button("add model").clicked() {
        scene_object.model_component = Some(ModelComponent::default());
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
