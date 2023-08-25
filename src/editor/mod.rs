use crate::SceneObjectId;

pub mod asset_browser;
pub mod hierarchy;
pub mod inspector;

pub struct Editor {
    selected_scene_object_id: SceneObjectId,
    file_browser_open: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            selected_scene_object_id: SceneObjectId::EMPTY,
            file_browser_open: false,
        }
    }
}
