use crate::{Camera, Sun, Vertex};

use super::{
    renderer::{self, Renderer},
    RenderPass,
};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

pub const SHADOW_PASS_TEXTURE_SIZE: u32 = 1024;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunUniform {
    mvp: Mat4,
}

impl SunUniform {
    pub fn update(&mut self, camera: &Camera, sun: &Sun) {
        let view = Mat4::look_at_rh(sun.inverse_direction, Vec3::ZERO, Vec3::Y);
        self.mvp = sun.projection * view * camera.transform;
    }
}

pub struct ShadowRenderPass {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,

    sun_uniform: SunUniform,
    sun_buffer: wgpu::Buffer,
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

        let mut sun_uniform = SunUniform::default();
        sun_uniform.update(&renderer.camera, &renderer.sun);

        let sun_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shadow pass bind group sun buffer"),
                contents: bytemuck::cast_slice(&[sun_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("shadow pass bind group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sun_buffer.as_entire_binding(),
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
            bind_group_layout,
            bind_group,
            pipeline,
            texture,
            texture_view,
            sun_uniform,
            sun_buffer,
        }
    }
}

impl RenderPass for ShadowRenderPass {
    fn prepare(&mut self, renderer: &Renderer) {
        self.sun_uniform.update(&renderer.camera, &renderer.sun);

        renderer.queue.write_buffer(
            &self.sun_buffer,
            0,
            bytemuck::cast_slice(&[self.sun_uniform]),
        );
    }

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

        for object in &renderer.scene_objects {
            let mesh = renderer.meshes.get(&object.mesh_handle).unwrap();
            let vertex_buffer = renderer.buffers.get(&mesh.vertex_buffer_handle).unwrap();
            let index_buffer = renderer.buffers.get(&mesh.index_buffer_handle).unwrap();

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(
                mesh.index_offset as u32..(mesh.index_offset + mesh.index_count) as u32,
                mesh.vertex_offset as i32,
                0..1,
            );
        }
    }

    fn cleanup(&mut self, _renderer: &Renderer) {}
}
