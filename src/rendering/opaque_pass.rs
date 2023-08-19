use super::{renderer::Renderer, RenderPass};

pub struct OpaqueRenderPass {}

impl OpaqueRenderPass {
    pub fn new(renderer: &Renderer) -> Self {
        Self {}
    }
}

impl RenderPass for OpaqueRenderPass {
    fn prepare(&mut self, _renderer: &Renderer) {}

    fn render(
        &mut self,
        renderer: &Renderer,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("opaque render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_bind_group(0, &renderer.global_bind_group, &[]);

        
    }

    fn cleanup(&mut self, _renderer: &Renderer) {}
}
