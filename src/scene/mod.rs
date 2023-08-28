use crate::{
    components::{camera::CameraComponent, light::LightComponent},
    Id,
};
use glam::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use self::scene_object::SceneObject;

pub type SceneObjectId = Id;

pub const DEFAULT_SCENE_PATH: &'static str = "./scene.data";

pub mod scene_object;

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub scene_objects: Vec<SceneObject>,
    pub camera_scene_object_id: SceneObjectId,
    pub sun_scene_object_id: SceneObjectId,
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
        self.scene_objects.push(SceneObject::default());

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
            .find(|scene_object| scene_object.id() == scene_object_id)
    }

    pub fn get_mut(&mut self, scene_object_id: SceneObjectId) -> Option<&mut SceneObject> {
        self.scene_objects
            .iter_mut()
            .find(|scene_object| scene_object.id() == scene_object_id)
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
            .position(|so| so.id() == scene_object_id)
            .unwrap();

        self.scene_objects.swap_remove(index);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self {
            scene_objects: Default::default(),
            camera_scene_object_id: Default::default(),
            sun_scene_object_id: Default::default(),
        };

        let camera = scene.add_scene_object();
        camera.transform_component.position = Vec3::new(0.0, 0.0, -2.0);
        camera.camera_component = Some(CameraComponent::default());
        camera.name = String::from("Camera");
        scene.camera_scene_object_id = camera.id();

        let sun = scene.add_scene_object();
        sun.transform_component.position = Vec3::new(5.0, 5.0, 5.0);
        sun.light_component = Some(LightComponent::default());
        sun.name = String::from("Sun");
        scene.sun_scene_object_id = sun.id();

        scene
    }
}
