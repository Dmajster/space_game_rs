use wgpu::Color;

use super::RenderPass;

pub struct OpaqueRenderPass {}

impl OpaqueRenderPass {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderPass for OpaqueRenderPass {
    fn prepare(&mut self, renderer: &super::renderer::Renderer) {

    }

    fn render(
        &mut self,
        renderer: &super::renderer::Renderer,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("opaque render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLUE),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    }

    fn cleanup(&mut self, renderer: &super::renderer::Renderer) {

    }
}
