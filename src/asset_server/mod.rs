use self::asset_id::AssetId;
use crate::rendering::{Material, Mesh, Model, Texture};
use serde::{Deserialize, Serialize};
use std::{
    any::type_name,
    cell::{Ref, RefCell, RefMut},
    fs::{self},
    ops::{Deref, DerefMut},
    path::Path,
    rc::Rc,
    slice::Iter,
};

pub mod asset_id;

pub const DEFAULT_PATH: &'static str = "./assets_server.data";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Asset<T> {
    id: AssetId<T>,
    pub metadata: AssetMetadata,
    pub asset: T,
}

impl<T> Deref for Asset<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.asset
    }
}

impl<T> DerefMut for Asset<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.asset
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub name: Option<String>,
}

impl<T> Asset<T> {
    pub fn new(asset: T, mut metadata: AssetMetadata) -> Self {
        let typed_id = AssetId::<T>::new();

        metadata.name = if let Some(name) = metadata.name {
            Some(name)
        } else {
            Some(format!(
                "{}_{}",
                type_name::<T>().split("::").last().unwrap(),
                typed_id.id().0
            ))
        };

        Self {
            id: typed_id,
            metadata,
            asset,
        }
    }

    pub fn id(&self) -> AssetId<T> {
        self.id
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssetStore<T> {
    assets: Vec<Asset<T>>,
}

impl<T> AssetStore<T> {
    pub fn add(&mut self, asset: T, metadata: AssetMetadata) -> &mut Asset<T> {
        self.assets.push(Asset::new(asset, metadata));
        self.assets.last_mut().unwrap()
    }

    pub fn remove_at_index(&mut self, index: usize) {
        self.assets.remove(index);
    }

    pub fn get(&self, asset_id: &AssetId<T>) -> Option<&Asset<T>> {
        self.assets.iter().find(|asset| asset.id() == *asset_id)
    }

    pub fn get_at_index(&self, index: usize) -> Option<&Asset<T>> {
        self.assets.get(index)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn iter(&self) -> Iter<'_, Asset<T>> {
        self.assets.iter()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssetServer {
    models: Rc<RefCell<AssetStore<Model>>>,
    meshes: Rc<RefCell<AssetStore<Mesh>>>,
    textures: Rc<RefCell<AssetStore<Texture>>>,
    materials: Rc<RefCell<AssetStore<Material>>>,
}

impl AssetServer {
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

    pub fn models(&self) -> Ref<'_, AssetStore<Model>> {
        self.models.borrow()
    }

    pub fn models_mut(&self) -> RefMut<'_, AssetStore<Model>> {
        self.models.borrow_mut()
    }

    pub fn meshes(&self) -> Ref<'_, AssetStore<Mesh>> {
        self.meshes.borrow()
    }

    pub fn meshes_mut(&self) -> RefMut<'_, AssetStore<Mesh>> {
        self.meshes.borrow_mut()
    }

    pub fn textures(&self) -> Ref<'_, AssetStore<Texture>> {
        self.textures.borrow()
    }

    pub fn textures_mut(&self) -> RefMut<'_, AssetStore<Texture>> {
        self.textures.borrow_mut()
    }

    pub fn materials(&self) -> Ref<'_, AssetStore<Material>> {
        self.materials.borrow()
    }

    pub fn materials_mut(&self) -> RefMut<'_, AssetStore<Material>> {
        self.materials.borrow_mut()
    }
}
