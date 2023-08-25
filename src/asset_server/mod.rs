use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::rendering::{Mesh, MeshDescriptor, MeshId};

pub const DEFAULT_PATH: &'static str = "./assets_server.data";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssetServer {
    meshes: Vec<Mesh>,
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

    pub fn add_mesh_from_desc(&mut self, descriptor: MeshDescriptor) -> &mut Mesh {
        self.meshes.push(Mesh::new(descriptor));
        self.meshes.last_mut().unwrap()
    }

    pub fn get_mesh(&self, mesh_id: &MeshId) -> Option<&Mesh> {
        self.meshes.iter().find(|mesh| mesh.id() == *mesh_id)
    }

    pub fn get_mesh_at_index(&self, index: usize) -> Option<&Mesh> {
        self.meshes.get(index)
    }

    pub fn get_meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }

    pub fn remove_mesh_at_index(&mut self, index: usize) {
        self.meshes.remove(index);
    }
}
