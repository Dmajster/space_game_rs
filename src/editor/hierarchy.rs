use egui::{CollapsingHeader, Ui, Window};

use crate::app::{Res, ResMut};
use crate::editor::Editor;
use crate::scene::{SceneObject, SceneObjectId};
use crate::Scene;

#[derive(Default)]
struct HierarchyChanges {
    add_scene_objects: Vec<SceneObjectId>, // List of id's to whom children are added
    remove_scene_objects: Vec<SceneObjectId>, // List of id's of removed scene objects
}

pub fn update(context: Res<egui::Context>, scene: ResMut<Scene>, editor: ResMut<Editor>) {
    let context = context.get();
    let editor = editor.get_mut();
    let mut changes = HierarchyChanges::default();

    Window::new("Hierarchy")
        .default_width(512.0)
        .show(&context, |ui| {
            for scene_object in &scene.get().scene_objects {
                if scene_object.parent_id == SceneObjectId::EMPTY {
                    ui_tree_recursive(ui, 0, &scene.get(), scene_object, &mut changes);
                }
            }
        })
        .unwrap()
        .response
        .context_menu(|ui| {
            ui_tree_context_menu(ui, SceneObjectId::EMPTY, &mut changes);
        });

    let mut scene = scene.get_mut();
    for parent_id in changes.add_scene_objects {
        println!("before new: {:#?}", scene);
        
        let new_scene_object_id = scene.add_scene_object().id();
        scene.reparent(new_scene_object_id, parent_id);
        println!("after new: {:#?}", scene);
    }

    for removed_id in changes.remove_scene_objects {
        println!("before remove: {:#?}", scene);
        scene.remove_scene_object(removed_id);
        println!("after remove: {:#?}", scene);
    }
}

fn ui_tree_recursive(
    ui: &mut Ui,
    depth: usize,
    scene: &Scene,
    scene_object: &SceneObject,
    changes: &mut HierarchyChanges,
) {
    CollapsingHeader::new(scene_object.name.as_str())
        .default_open(false)
        .id_source(scene_object.id().0)
        .show(ui, |ui| {
            for child_id in &scene_object.children {
                let child = scene.get(*child_id).unwrap();

                ui_tree_recursive(ui, depth + 1, scene, child, changes);
            }
        })
        .header_response
        .context_menu(|ui| {
            ui_tree_context_menu(ui, scene_object.id(), changes);
        });
}

fn ui_tree_context_menu(
    ui: &mut Ui,
    selected_scene_object_id: SceneObjectId,
    changes: &mut HierarchyChanges,
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
