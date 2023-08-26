use crate::{
    asset_server::AssetId,
    components::{mesh_component::MeshComponent, Component},
};
use glam::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::SceneObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneObject {
    pub name: String,
    id: SceneObjectId,
    pub parent_id: SceneObjectId,
    pub children: Vec<SceneObjectId>,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub mesh_id: AssetId,

    pub components: Vec<Box<dyn Component>>,
}

impl SceneObject {
    pub fn calculate_transform(&self) -> Mat4 {
        let cr = (self.rotation.x.to_radians() * 0.5).cos();
        let sr = (self.rotation.x.to_radians() * 0.5).sin();
        let cp = (self.rotation.y.to_radians() * 0.5).cos();
        let sp = (self.rotation.y.to_radians() * 0.5).sin();
        let cy = (self.rotation.z.to_radians() * 0.5).cos();
        let sy = (self.rotation.z.to_radians() * 0.5).sin();

        let rotation = Quat::from_xyzw(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        );

        Mat4::from_scale_rotation_translation(self.scale, rotation, self.position)
    }

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
            mesh_id: AssetId::EMPTY,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            components: vec![Box::new(MeshComponent::default())],
        }
    }
}
