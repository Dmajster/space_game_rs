use glam::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::components::{camera::CameraComponent, light::LightComponent, model::ModelComponent, transform::TransformComponent};

use super::SceneObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneObject {
    pub name: String,
    id: SceneObjectId,
    pub parent_id: SceneObjectId,
    pub children: Vec<SceneObjectId>,
    pub transform_component: TransformComponent,
    pub model_component: Option<ModelComponent>,
    pub light_component: Option<LightComponent>,
    pub camera_component: Option<CameraComponent>,
}

impl SceneObject {
    pub fn id(&self) -> SceneObjectId {
        self.id
    }
}

impl Default for SceneObject {
    fn default() -> Self {
        Self {
            name: String::from("Scene object"),
            id: SceneObjectId::new(),
            parent_id: SceneObjectId::EMPTY,
            children: vec![],
            transform_component: TransformComponent::default(),
            model_component: None,
            light_component: None,
            camera_component: None,
        }
    }
}
