use egui::{CollapsingHeader, Ui, Window};

use crate::app::{Res, ResMut};
use crate::editor::Editor;
use crate::scene::scene_object::SceneObject;
use crate::scene::{self, SceneObjectId};
use crate::Scene;

#[derive(Default)]
struct SceneHierarchyChanges {
    add_scene_objects: Vec<SceneObjectId>, // List of id's to whom children are added
    remove_scene_objects: Vec<SceneObjectId>, // List of id's of removed scene objects
    selected_scene_object_id: SceneObjectId,
}

pub fn update(context: Res<egui::Context>, scene: ResMut<Scene>, editor: ResMut<Editor>) {
    let context = context.get();
    let mut editor = editor.get_mut();
    let mut changes = SceneHierarchyChanges::default();
    let mut scene = scene.get_mut();

    Window::new("Scene hierarchy")
        .min_width(512.0)
        .show(&context, |ui| {
            if ui.button("save scene").clicked() {
                scene.write_to_file(&scene::DEFAULT_SCENE_PATH)
            }

            for scene_object in &scene.scene_objects {
                if scene_object.parent_id == SceneObjectId::EMPTY {
                    ui_tree_recursive(ui, 0, &scene, scene_object, &mut changes);
                }
            }
        })
        .unwrap()
        .response
        .context_menu(|ui| {
            ui_tree_context_menu(ui, SceneObjectId::EMPTY, &mut changes);
        });

    for parent_id in changes.add_scene_objects {
        let new_scene_object_id = scene.add_scene_object().id();
        scene.reparent(new_scene_object_id, parent_id);
    }

    for removed_id in changes.remove_scene_objects {
        scene.remove_scene_object(removed_id);
    }

    if changes.selected_scene_object_id != SceneObjectId::EMPTY {
        editor.selected_scene_object_id = changes.selected_scene_object_id;
    }
}

fn ui_tree_recursive(
    ui: &mut Ui,
    depth: usize,
    scene: &Scene,
    scene_object: &SceneObject,
    changes: &mut SceneHierarchyChanges,
) {
    let header_response = CollapsingHeader::new(scene_object.name.as_str())
        .default_open(false)
        .id_source(scene_object.id().0)
        .show(ui, |ui| {
            for child_id in &scene_object.children {
                let child = scene.get(*child_id).unwrap();

                ui_tree_recursive(ui, depth + 1, scene, child, changes);
            }
        })
        .header_response;

    if header_response.clicked() {
        changes.selected_scene_object_id = scene_object.id();
    }

    header_response.context_menu(|ui| {
        ui_tree_context_menu(ui, scene_object.id(), changes);
    });
}

fn ui_tree_context_menu(
    ui: &mut Ui,
    selected_scene_object_id: SceneObjectId,
    changes: &mut SceneHierarchyChanges,
) {
    if ui.button("add scene object").clicked() {
        changes.add_scene_objects.push(selected_scene_object_id);
        ui.close_menu();
    }

    if selected_scene_object_id != SceneObjectId::EMPTY {
        if ui.button("remove").clicked() {
            changes.remove_scene_objects.push(selected_scene_object_id);
            ui.close_menu();
        }
    }
}
