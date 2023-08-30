use glam::Vec3;
use serde::{Serialize, Deserialize};

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderLight {
    pub position: Vec3,
    pub luminous_intensity: f32,
    //
    pub direction: Vec3,
    pub inner_angle: f32,
    //
    pub color: Vec3,
    pub outer_angle: f32,
    //
    pub falloff_radius: f32,
    pub ty: i32,
    pub unused0: f32,
    pub unused1: f32,
}
