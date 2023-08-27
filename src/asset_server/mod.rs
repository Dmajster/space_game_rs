use std::{
    any::type_name,
    fs::{self},
    ops::{Deref, DerefMut},
    path::Path,
    slice::Iter,
};

use serde::{Deserialize, Serialize};

use crate::{
    rendering::{Mesh, Model, Texture},
    Id,
};

pub const DEFAULT_PATH: &'static str = "./assets_server.data";

pub type AssetId = Id;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Asset<T> {
    id: AssetId,
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
        let id = AssetId::new();

        metadata.name = if let Some(name) = metadata.name {
            Some(name)
        } else {
            Some(format!(
                "{}_{}",
                type_name::<T>().split("::").last().unwrap(),
                id.0
            ))
        };

        Self {
            id,
            metadata,
            asset,
        }
    }

    pub fn id(&self) -> AssetId {
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

    pub fn get(&self, asset_id: &AssetId) -> Option<&Asset<T>> {
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
    pub models: AssetStore<Model>,
    pub meshes: AssetStore<Mesh>,
    pub textures: AssetStore<Texture>,
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
}
