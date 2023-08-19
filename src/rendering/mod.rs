use wgpu::{CommandEncoder, TextureView};

use self::renderer::Renderer;

pub mod renderer;

pub mod shadow_pass;
pub mod opaque_pass;
pub mod egui_pass;

pub trait RenderPass {
    fn prepare(&mut self, renderer: &Renderer);

    fn render(&mut self, renderer: &Renderer, encoder: &mut CommandEncoder, view: &TextureView);

    fn cleanup(&mut self, renderer: &Renderer);
}
