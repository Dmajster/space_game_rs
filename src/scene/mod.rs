use crate::{rendering::MeshId, Id};
use glam::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub type SceneObjectId = Id;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scene {
    pub scene_objects: Vec<SceneObject>,
}

impl Scene {
    pub fn read_from_file_or_new<P>(path: &P) -> Self
    where
        P: AsRef<Path>,
    {
        if let Ok(bytes) = fs::read(path) {
            let decompressed_bytes = lz4_flex::decompress_size_prepended(&bytes).unwrap();

            bincode::deserialize::<Self>(&decompressed_bytes).unwrap()
        } else {
            Default::default()
        }
    }

    pub fn write_to_file<P>(&self, path: &P)
    where
        P: AsRef<Path>,
    {
        let bytes = bincode::serialize::<Self>(&self).unwrap();

        let compressed_bytes = lz4_flex::compress_prepend_size(&bytes);

        fs::write(path, compressed_bytes).unwrap();
    }

    pub fn add_scene_object(&mut self) -> &mut SceneObject {
        self.scene_objects.push(SceneObject {
            name: String::from("Scene object"),
            id: SceneObjectId::new(),
            parent_id: SceneObjectId::EMPTY,
            children: vec![],
            mesh_id: MeshId::EMPTY,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        });

        self.scene_objects.last_mut().unwrap()
    }

    pub fn reparent(&mut self, child_id: SceneObjectId, new_parent_id: SceneObjectId) {
        let child = self.get_mut(child_id).unwrap();
        let old_parent_id = child.parent_id;
        child.parent_id = new_parent_id;

        if old_parent_id != SceneObjectId::EMPTY {
            self.remove_child(old_parent_id, child_id)
        }

        if new_parent_id != SceneObjectId::EMPTY {
            let new_parent = self.get_mut(new_parent_id).unwrap();
            new_parent.children.push(child_id);
        }
    }

    fn remove_child(&mut self, parent_id: SceneObjectId, removed_child_id: SceneObjectId) {
        let parent: &mut SceneObject = self.get_mut(parent_id).unwrap();
        let child_index = parent
            .children
            .iter()
            .position(|id| removed_child_id == *id)
            .unwrap();
        parent.children.swap_remove(child_index);
    }

    pub fn get(&self, scene_object_id: SceneObjectId) -> Option<&SceneObject> {
        self.scene_objects
            .iter()
            .find(|scene_object| scene_object.id == scene_object_id)
    }

    pub fn get_mut(&mut self, scene_object_id: SceneObjectId) -> Option<&mut SceneObject> {
        self.scene_objects
            .iter_mut()
            .find(|scene_object| scene_object.id == scene_object_id)
    }

    pub fn remove_scene_object(&mut self, scene_object_id: SceneObjectId) {
        let scene_object = self.get(scene_object_id).unwrap();
        let parent_id = scene_object.parent_id;
        let children = scene_object.children.clone();

        // If scene object had a parent remove it from it's children
        if parent_id != SceneObjectId::EMPTY {
            self.remove_child(parent_id, scene_object_id);
        }

        // Destroy all the children
        for child_id in children {
            self.remove_scene_object(child_id);
        }

        // Get index and remove from scene objects
        let index = self
            .scene_objects
            .iter()
            .position(|so| so.id == scene_object_id)
            .unwrap();

        self.scene_objects.swap_remove(index);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SceneObject {
    pub name: String,
    id: SceneObjectId,
    pub parent_id: SceneObjectId,
    pub children: Vec<SceneObjectId>,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub mesh_id: MeshId,
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
