use crate::editor::inspector::EditorInspector;
use std::fmt::Debug;

pub mod mesh_component;

#[typetag::serde(tag = "component")]
pub trait Component: Debug + EditorInspector {}