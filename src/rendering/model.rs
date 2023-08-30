use super::{helpers::Handle, material::Material};
use crate::asset_server::asset_id::AssetId;
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv: Vec2,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tangents: Vec<Vec3>,
    pub bitangents: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RenderMesh {
    pub vertex_buffer_handle: Handle<wgpu::Buffer>,
    pub vertex_offset: usize,
    pub vertex_count: usize,
    pub index_buffer_handle: Handle<wgpu::Buffer>,
    pub index_offset: usize,
    pub index_count: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Model {
    pub mesh_ids: Vec<AssetId<Mesh>>,
    pub material_ids: Vec<AssetId<Material>>,
}
