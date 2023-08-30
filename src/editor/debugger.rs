use egui::Window;

use crate::app::Res;

pub fn update(context: Res<egui::Context>) {
    let context = context.get();

    Window::new("Debugger")
        .min_width(512.0)
        .show(&context, |ui| puffin_egui::profiler_ui(ui));
}
