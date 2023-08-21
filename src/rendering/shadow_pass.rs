use crate::{Camera, Sun};

use super::{
    renderer::{self, RenderInstance, Renderer, Vertex},
    RenderPass,
};
use glam::{Mat4, Vec3};

pub const SHADOW_PASS_TEXTURE_SIZE: u32 = 2048;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunUniform {
    mvp: Mat4,
}

impl SunUniform {
    pub fn update(&mut self, camera: &Camera, sun: &Sun) {
        let view = Mat4::look_at_rh(sun.inverse_direction, Vec3::ZERO, Vec3::Y);
        self.mvp = sun.projection * view;
    }
}

pub struct ShadowRenderPass {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    texture_view: wgpu::TextureView,
}

impl ShadowRenderPass {
    pub fn new(renderer: &mut Renderer) -> Self {
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

        renderer
            .render_pass_resources
            .insert("shadow depth", texture);

        let texture_view = renderer
            .render_pass_resources
            .get("shadow depth")
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("shadow pass bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("shadow pass bind group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: renderer.sun_buffer.as_entire_binding(),
                }],
            });

        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shadow pass pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout], //&bind_group_layout
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
                    buffers: &[
                    // Vertex
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
                    },
                    // Instance
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<RenderInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8=> Float32x4],
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
            bind_group,
            pipeline,
            texture_view,
        }
    }
}

impl RenderPass for ShadowRenderPass {
    fn prepare(&mut self, renderer: &Renderer) {}

    fn render(
        &mut self,
        renderer: &Renderer,
        encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(1, renderer.scene_object_instances.slice(..));

        for (index, object) in renderer.scene_objects.iter().enumerate() {
            let mesh = renderer.meshes.get(&object.mesh_handle).unwrap();
            let vertex_buffer = renderer
                .mesh_buffers
                .get(&mesh.vertex_buffer_handle)
                .unwrap();
            let index_buffer = renderer
                .mesh_buffers
                .get(&mesh.index_buffer_handle)
                .unwrap();

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(
                mesh.index_offset as u32..(mesh.index_offset + mesh.index_count) as u32,
                mesh.vertex_offset as i32,
                index as u32..(index + 1) as u32,
            );
        }
    }

    fn cleanup(&mut self, _renderer: &Renderer) {}
}
