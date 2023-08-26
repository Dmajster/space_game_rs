use egui::{DragValue, Ui, Window};

use crate::app::App;
use crate::editor::Editor;
use crate::scene::SceneObjectId;
use crate::Scene;

pub trait EditorInspector {
    fn draw(&mut self, ui: &mut Ui, app: &mut App);
}

pub fn update(app: &mut App) {
    let context = app.get_resource::<egui::Context>().unwrap();
    let context = context.get();

    let editor = app.get_resource::<Editor>().unwrap();
    let editor = editor.get();

    let scene = app.get_resource_mut::<Scene>().unwrap();
    let mut scene = scene.get_mut();

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

            for component in &mut sobj.components {
                ui.heading(component.typetag_name());
                ui.add_space(8.0);

                component.draw(ui, app);

                ui.separator();
            }

            // ui.heading("Mesh");
            // ui.add_space(8.0);

            // ui.separator();
        });
}
