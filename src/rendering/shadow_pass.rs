use super::{
    renderer::{self, Renderer, Vertex},
    RenderPass,
};

pub const SHADOW_PASS_TEXTURE_SIZE: u32 = 1024;

pub struct ShadowRenderPass {
    // bind_group_layout: wgpu::BindGroupLayout,
    // bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

impl ShadowRenderPass {
    pub fn new(renderer: &Renderer) -> Self {
        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow pass texture"),
            size: wgpu::Extent3d {
                width: SHADOW_PASS_TEXTURE_SIZE,
                height: SHADOW_PASS_TEXTURE_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer::DEPTH_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // let bind_group_layout =
        //     renderer
        //         .device
        //         .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //             label: Some("shadow pass bind group layout"),
        //             entries: &[],
        //         });

        // let bind_group = renderer
        //     .device
        //     .create_bind_group(&wgpu::BindGroupDescriptor {
        //         label: Some("shadow pass bind group"),
        //         layout: &bind_group_layout,
        //         entries: &[],
        //     });

        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shadow pass pipeline layout"),
                    bind_group_layouts: &[
                        &renderer.bind_group_layout
                    ], //&bind_group_layout
                    push_constant_ranges: &[],
                });

        let shader = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shadow pass shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../assets/shaders/shadow_pass.wgsl").into(),
                ),
            });

        let pipeline = renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shadow pass render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: renderer::DEPTH_TEXTURE_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: None,
                multiview: None,
            });

        Self {
            // bind_group_layout,
            // bind_group,
            pipeline,
            texture,
            texture_view,
        }
    }
}

impl RenderPass for ShadowRenderPass {
    fn prepare(&mut self, _renderer: &super::renderer::Renderer) {}

    fn render(
        &mut self,
        renderer: &super::renderer::Renderer,
        encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
    ) {
        let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        shadow_pass.set_pipeline(&self.pipeline);
        shadow_pass.set_bind_group(0, &renderer.bind_group, &[]);
        // shadow_pass.set_bind_group(1, &self.bind_group, &[]);
    }

    fn cleanup(&mut self, _renderer: &super::renderer::Renderer) {}
}
