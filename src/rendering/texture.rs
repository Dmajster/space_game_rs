use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub bytes: Vec<u8>,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            format: wgpu::TextureFormat::Rgba32Float,
            bytes: Default::default(),
        }
    }
}
