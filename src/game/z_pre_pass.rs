use crate::{app::{Res, ResMut}, rendering::{Renderer, model::Vertex, RenderInstance, self, RenderingRecorder},game::Game, asset_server::{AssetServer, asset_id::AssetId}, scene::Scene};


pub struct ZPreRenderPass {
    pipeline: wgpu::RenderPipeline,
}

impl ZPreRenderPass {
    pub fn new(renderer: &mut Renderer, global_bind_group_layout: &wgpu::BindGroupLayout) -> Self {        
        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shadow pass pipeline layout"),
                    bind_group_layouts: &[&global_bind_group_layout], //&bind_group_layout
                    push_constant_ranges: &[],
                });

        let shader = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shadow pass shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../assets/shaders/z_pre_pass.wgsl").into(),
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
                        attributes: &wgpu::vertex_attr_array![
                            // Position
                            0 => Float32x3, 
                            // Normal
                            1 => Float32x3,
                            // Tangent
                            2 => Float32x3,
                            // Bitangent
                            3 => Float32x3,
                            // Uv
                            4 => Float32x2,
                        ],
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
                fragment: None,
                multiview: None,
            });

        Self {
            pipeline,
        }
    }
}


pub fn render(
    game: Res<Game>,
    scene: Res<Scene>,
    renderer: Res<Renderer>,
    rendering_recorder: ResMut<Option<RenderingRecorder>>,
    asset_server: Res<AssetServer>,
) {
    let game = game.get();
    let scene = scene.get();
    let renderer = renderer.get();
    let asset_server = asset_server.get();
    let mut rendering_recorder = rendering_recorder.get_mut();
    let rendering_recorder = rendering_recorder.as_mut().unwrap();

    let mut render_pass =
        rendering_recorder
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("z pre pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &renderer.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

    render_pass.set_pipeline(&game.z_pre_pass.pipeline);
    render_pass.set_bind_group(0, &game.global_bind_group, &[]);
    render_pass.set_vertex_buffer(1, renderer.scene_object_instances.slice(..));

    let models = asset_server.models();

    for (index, scene_object) in scene.scene_objects.iter().enumerate() {
        if let Some(model_component) = &scene_object.model_component {
            if model_component.model_id == AssetId::EMPTY {
                continue;
            }

            let model = models.get(&model_component.model_id).unwrap();

            for mesh_id in &model.mesh_ids {
                if let Some(render_mesh) = renderer.get_render_mesh(mesh_id) {
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
}
