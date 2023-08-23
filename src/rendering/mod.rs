use wgpu::{CommandEncoder, TextureView};

use crate::App;

pub mod egui_pass;
pub mod opaque_pass;
pub mod renderer;
pub mod shadow_pass;

pub trait RenderPass {
    fn prepare(&mut self, app: &App);

    fn render(&mut self, app: &App, encoder: &mut CommandEncoder, view: &TextureView);

    fn cleanup(&mut self, app: &App);
}
