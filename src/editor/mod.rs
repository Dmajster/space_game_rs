use egui::{CentralPanel, Frame};
use egui_dock::{DockArea, Style, Tree};

use crate::{app::App, ui::Egui, SceneObjectId};

pub mod asset_browser;
pub mod hierarchy;
pub mod inspector;

pub struct Editor {
    selected_scene_object_id: SceneObjectId,
    file_browser_open: bool,

    tree: Tree<String>,
}

pub struct EditorTab {
    pub name: String,
    pub draw_fm: Box<dyn Fn(&mut App, &mut Egui, &mut Editor)>,
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

pub fn update(app: &mut App, egui: &mut Egui, editor: &mut Editor) {
    CentralPanel::default()
        .frame(Frame::central_panel(&egui.context.style()).inner_margin(0.))
        .show(&egui.context, |ui| {
            DockArea::new(&mut editor.tree)
                .style(Style::from_egui(egui.context.style().as_ref()))
                .show(&egui.context, &mut TabViewer {});
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
