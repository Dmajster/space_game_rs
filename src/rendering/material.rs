use super::texture::Texture;
use crate::asset_server::asset_id::AssetId;
use glam::Vec4;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(
    Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize,
)]
pub struct MaterialProperties {
    pub base_color_factor: Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub reflectance: f32,
    pub padding0: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Material {
    pub color_texture_id: Option<AssetId<Texture>>,
    pub normal_texture_id: Option<AssetId<Texture>>,
    pub metallic_roughness_texture_id: Option<AssetId<Texture>>, //TODO: split this into seperate textures

    pub material_properties: MaterialProperties,
}

pub struct RenderMaterial {
    pub bind_group: wgpu::BindGroup,
}
