use crate::{
    app::{Res, ResMut},
    game::Game,
    rendering::{self, RenderInstance, Renderer, RenderingRecorder, Vertex},
    Scene,
};

pub struct OpaqueRenderPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl OpaqueRenderPass {
    pub fn new(
        renderer: &Renderer,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        sun_buffer: &wgpu::Buffer,
    ) -> Self {
        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("opaque pass bind group layout"),
                    entries: &[
                        // Sun
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Depth sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            count: None,
                        },
                        // Shadow depth texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let shadow_depth_texture_view = &renderer
            .render_pass_resources
            .get("shadow depth")
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("opaque pass bind group"),
                layout: &bind_group_layout,
                entries: &[
                    // Sun
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            sun_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    // Depth sampler
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&renderer.sampler),
                    },
                    // Shadow depth texture
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&shadow_depth_texture_view),
                    },
                ],
            });

        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("opaque pass pipeline layout"),
                    bind_group_layouts: &[&global_bind_group_layout, &bind_group_layout], //&bind_group_layout
                    push_constant_ranges: &[],
                });

        let shader = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("opaque pass shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../assets/shaders/opaque_pass.wgsl").into(),
                ),
            });

        let pipeline = renderer
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("opaque pass render pipeline"),
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
                format: rendering::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: renderer.surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            multiview: None,
        });

        Self {
            pipeline,
            bind_group,
        }
    }
}

pub fn render(
    game: Res<Game>,
    scene: Res<Scene>,
    renderer: Res<Renderer>,
    rendering_recorder: ResMut<Option<RenderingRecorder>>,
) {
    let app = game.get();
    let scene = scene.get();
    let renderer = renderer.get();
    let mut rendering_recorder = rendering_recorder.get_mut();
    let rendering_recorder = rendering_recorder.as_mut().unwrap();

    let mut render_pass =
        rendering_recorder
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaque render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &rendering_recorder.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.52,
                            g: 0.80,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &renderer.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

    render_pass.set_pipeline(&app.opaque_pass.pipeline);
    render_pass.set_bind_group(0, &app.global_bind_group, &[]);
    render_pass.set_bind_group(1, &app.opaque_pass.bind_group, &[]);
    render_pass.set_vertex_buffer(1, renderer.scene_object_instances.slice(..));

    for (index, scene_object) in scene.scene_objects.iter().enumerate() {
        if let Some(mesh_component) = &scene_object.mesh_component {
            if let Some(render_mesh) = renderer.get_render_mesh(&mesh_component.mesh_id) {
                let vertex_buffer = renderer
                    .mesh_buffers
                    .get(&render_mesh.vertex_buffer_handle)
                    .unwrap();
                let index_buffer = renderer
                    .mesh_buffers
                    .get(&render_mesh.index_buffer_handle)
                    .unwrap();

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(
                    render_mesh.index_offset as u32
                        ..(render_mesh.index_offset + render_mesh.index_count) as u32,
                    render_mesh.vertex_offset as i32,
                    index as u32..(index + 1) as u32,
                );
            }
        }
    }
}
