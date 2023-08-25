use egui::{CentralPanel, Frame};
use egui_dock::{DockArea, Style, Tree};

use crate::{
    app::{Res, ResMut},
    SceneObjectId,
};

// pub mod asset_browser;
// pub mod hierarchy;
// pub mod inspector;

pub struct Editor {
    selected_scene_object_id: SceneObjectId,
    file_browser_open: bool,

    tree: Tree<String>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(vec!["tab1".to_owned(), "tab2".to_owned()]),
            selected_scene_object_id: SceneObjectId::EMPTY,
            file_browser_open: false,
        }
    }
}

pub fn update(context: Res<egui::Context>, editor: ResMut<Editor>) {
    let context = context.get();

    CentralPanel::default()
        .frame(Frame::central_panel(&context.style()).inner_margin(0.))
        .show(&context, |_| {
            DockArea::new(&mut editor.get_mut().tree)
                .style(Style::from_egui(context.style().as_ref()))
                .show(&context, &mut TabViewer {});
        });
}

struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}
