use crate::{
    app::{Res, ResMut},
    asset_server::{asset_id::AssetId, AssetServer},
    scene::{Scene, SceneObjectId},
};

use egui::epaint::ahash::HashMap;
use glam::Vec4;
use std::{cell::RefCell, collections::BTreeMap, iter};
use wgpu::util::DeviceExt;
use winit::window::Window;

use self::{
    helpers::Pool,
    material::{Material, RenderMaterial},
    model::{Mesh, RenderMesh, Vertex},
    texture::Texture,
};

pub mod helpers;
pub mod light;
pub mod material;
pub mod model;
pub mod texture;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInstance {
    pub model_matrix_0: Vec4,
    pub model_matrix_1: Vec4,
    pub model_matrix_2: Vec4,
    pub model_matrix_3: Vec4,
}

pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const SCENE_OBJECT_INSTANCES_BUFFER_SIZE: u64 = 20 * 1024 * 1024; //20MB

pub const MAX_LIGHTS_COUNT: u64 = 100;

// Rename this to low level renderer or gpu interface?
pub struct Renderer<'renderer> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub surface_capabilities: wgpu::SurfaceCapabilities,
    pub surface_format: wgpu::TextureFormat,
    pub surface_configuration: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub render_pass_resources: HashMap<&'renderer str, wgpu::Texture>,

    pub render_materials: BTreeMap<AssetId<Material>, RenderMaterial>,
    missing_render_material_ids: RefCell<Vec<AssetId<Material>>>,

    pub render_meshes: BTreeMap<AssetId<Mesh>, RenderMesh>,
    missing_render_mesh_ids: RefCell<Vec<AssetId<Mesh>>>,

    pub scene_object_instances: wgpu::Buffer,

    pub mesh_buffers: Pool<wgpu::Buffer>,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,

    pub filtrable_sampler: wgpu::Sampler,
    pub comparison_sampler: wgpu::Sampler,

    pub material_bind_group_layout: wgpu::BindGroupLayout,

    // RENDER OBJECT CACHING. Has no reclaiming so it grows infinitely with each new variant...
    // Might be a problem. Reclaiming would require tracking object usage.
    _bind_group_cache: BTreeMap<wgpu::BindGroupDescriptor<'renderer>, wgpu::BindGroup>,
    _bind_group_layout_cache:
        BTreeMap<wgpu::BindGroupLayoutDescriptor<'renderer>, wgpu::BindGroupLayout>,
    _pipeline_layout_cache:
        BTreeMap<wgpu::PipelineLayoutDescriptor<'renderer>, wgpu::PipelineLayout>,
    _render_pipeline_cache:
        BTreeMap<wgpu::RenderPipelineDescriptor<'renderer>, wgpu::RenderPipeline>,
}

impl<'renderer> Renderer<'renderer> {
    pub fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ))
        .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_configuration);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: surface_configuration.width,
                height: surface_configuration.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[DEPTH_TEXTURE_FORMAT],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let filtrable_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("filtrable sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: None,
            ..Default::default()
        });

        let comparison_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("comparison sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::Greater),
            ..Default::default()
        });

        let scene_object_instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene object instances"),
            size: SCENE_OBJECT_INSTANCES_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("material bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Color texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Normal texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Metallic roughness texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        Self {
            surface,
            surface_capabilities,
            surface_format,
            surface_configuration,
            instance,
            adapter,
            device,
            queue,

            render_pass_resources: Default::default(),

            depth_texture,
            depth_texture_view,

            filtrable_sampler,
            comparison_sampler,

            render_materials: Default::default(),
            missing_render_material_ids: RefCell::new(Vec::new()),

            render_meshes: Default::default(),
            missing_render_mesh_ids: RefCell::new(Vec::new()),

            mesh_buffers: Default::default(),

            material_bind_group_layout,

            _bind_group_cache: Default::default(),
            _bind_group_layout_cache: Default::default(),
            _pipeline_layout_cache: Default::default(),
            _render_pipeline_cache: Default::default(),
            scene_object_instances,
        }
    }

    pub fn get_render_mesh(&self, mesh_id: &AssetId<Mesh>) -> Option<&RenderMesh> {
        if *mesh_id == AssetId::EMPTY {
            return None;
        }

        if let Some(render_mesh) = self.render_meshes.get(&mesh_id) {
            Some(render_mesh)
        } else {
            self.missing_render_mesh_ids
                .borrow_mut()
                .push(mesh_id.clone());
            None
        }
    }

    pub fn get_render_material(&self, material_id: &AssetId<Material>) -> Option<&RenderMaterial> {
        if *material_id == AssetId::EMPTY {
            return None;
        }

        if let Some(render_material) = self.render_materials.get(&material_id) {
            Some(render_material)
        } else {
            self.missing_render_material_ids
                .borrow_mut()
                .push(material_id.clone());
            None
        }
    }

    pub fn create_render_meshes(&mut self, asset_server: &AssetServer) {
        let mut missing_render_mesh_ids = self.missing_render_mesh_ids.borrow_mut();
        let meshes = asset_server.meshes();

        while missing_render_mesh_ids.len() > 0 {
            let missing_render_mesh_id = missing_render_mesh_ids.pop().unwrap();
            let mesh = meshes.get(&missing_render_mesh_id).unwrap();

            let mut vertex_data = Vec::with_capacity(mesh.positions.len());

            for i in 0..mesh.positions.len() {
                vertex_data.push(Vertex {
                    position: mesh.positions[i],
                    normal: mesh.normals[i],
                    tangent: mesh.tangents[i],
                    bitangent: mesh.bitangents[i],
                    uv: mesh.uvs[i],
                });
            }

            let vertex_buffer_handle = self.mesh_buffers.add(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("vertex buffer"),
                    contents: bytemuck::cast_slice(vertex_data.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));

            let index_buffer_handle = self.mesh_buffers.add(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("index buffer"),
                    contents: bytemuck::cast_slice(mesh.indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ));

            self.render_meshes.insert(
                mesh.id(),
                RenderMesh {
                    vertex_buffer_handle,
                    vertex_offset: 0,
                    vertex_count: vertex_data.len(),
                    index_buffer_handle,
                    index_offset: 0,
                    index_count: mesh.indices.len(),
                },
            );
        }
    }

    pub fn create_render_materials(&mut self, asset_server: &AssetServer) {
        let mut missing_render_material_ids = self.missing_render_material_ids.borrow_mut();
        let materials = asset_server.materials();
        let textures = asset_server.textures();

        while missing_render_material_ids.len() > 0 {
            let missing_render_material_ids = missing_render_material_ids.pop().unwrap();
            let material = materials.get(&missing_render_material_ids).unwrap();

            let color_texture: Texture = {
                if let Some(texture_id) = material.color_texture_id {
                    let asset_texture = textures.get(&texture_id).unwrap();

                    asset_texture.asset.clone()
                } else {
                    Texture {
                        width: 1,
                        height: 1,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        bytes: vec![255, 255, 255, 255],
                    }
                }
            };

            let wgpu_color_texture = self.create_wgpu_texture(&color_texture);
            let wgpu_color_texture_view =
                wgpu_color_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let normal_texture = {
                if let Some(texture_id) = material.normal_texture_id {
                    let asset_texture = textures.get(&texture_id).unwrap();

                    asset_texture.asset.clone()
                } else {
                    Texture {
                        width: 1,
                        height: 1,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        bytes: vec![127, 127, 255, 255],
                    }
                }
            };

            let wgpu_normal_texture = self.create_wgpu_texture(&normal_texture);
            let wgpu_normal_texture_view =
                wgpu_normal_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let metallic_roughness_texture = {
                if let Some(texture_id) = material.metallic_roughness_texture_id {
                    let asset_texture = textures.get(&texture_id).unwrap();

                    asset_texture.asset.clone()
                } else {
                    Texture {
                        width: 1,
                        height: 1,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        bytes: vec![127, 127, 255, 255],
                    }
                }
            };

            let wgpu_metallic_roughness_texture =
                self.create_wgpu_texture(&metallic_roughness_texture);
            let wgpu_metallic_roughness_texture_view = wgpu_metallic_roughness_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let material_properties_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("material properties buffer"),
                        contents: bytemuck::cast_slice(&[material.material_properties]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("material bind group"),
                layout: &self.material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: material_properties_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.filtrable_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&wgpu_color_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&wgpu_normal_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(
                            &wgpu_metallic_roughness_texture_view,
                        ),
                    },
                ],
            });

            self.render_materials
                .insert(material.id(), RenderMaterial { bind_group });
        }
    }

    pub fn create_wgpu_texture(&self, texture: &Texture) -> wgpu::Texture {
        let extents = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };

        let wgpu_color_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("color texture"),
            size: extents,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &wgpu_color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * extents.width),
                rows_per_image: Some(extents.height),
            },
            extents,
        );

        wgpu_color_texture
    }
}

//TODO: Rework this function to go top to bottom. Currently it goes up recursively potentialy
// going over the same parent multiple times doing unnecessary recalculations.
pub fn update_scene_object_transforms(scene: Res<Scene>, renderer: Res<Renderer>) {
    let scene = scene.get();
    let renderer = renderer.get();

    let instances = scene
        .scene_objects
        .iter()
        .map(|mut scene_object| {
            let mut transform = scene_object.transform_component.build_transform_matrix();

            loop {
                if scene_object.parent_id != SceneObjectId::EMPTY {
                    let parent = scene
                        .scene_objects
                        .iter()
                        .find(|so| so.id() == scene_object.parent_id)
                        .unwrap();

                    transform = parent.transform_component.build_transform_matrix() * transform;

                    scene_object = parent;
                } else {
                    break;
                }
            }

            transform
        })
        .collect::<Vec<_>>();

    renderer.queue.write_buffer(
        &renderer.scene_object_instances,
        0,
        bytemuck::cast_slice(instances.as_slice()),
    );
}

pub struct RenderingRecorder {
    pub output: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

pub fn record(renderer: Res<Renderer>, rendering_recorder: ResMut<Option<RenderingRecorder>>) {
    let renderer = renderer.get();
    let mut rendering_recorder = rendering_recorder.get_mut();

    let output = {
        puffin_egui::puffin::profile_scope!("renderer.surface.get_current_texture()");

        renderer.surface.get_current_texture().unwrap()
    };

    let view = {
        puffin_egui::puffin::profile_scope!("output.texture.create_view()");

        output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    };

    let encoder = {
        puffin_egui::puffin::profile_scope!("renderer.device.create_command_encoder()");
        renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            })
    };

    *rendering_recorder = Some(RenderingRecorder {
        output,
        view,
        encoder,
    });

    puffin_egui::puffin::profile_scope!("render record");
}

pub fn present(renderer: Res<Renderer>, rendering_recorder_2: ResMut<Option<RenderingRecorder>>) {
    let renderer = renderer.get();

    let RenderingRecorder {
        output,
        view: _,
        encoder,
    } = rendering_recorder_2.replace(None).unwrap();

    puffin_egui::puffin::profile_scope!("render submit");
    renderer.queue.submit(iter::once(encoder.finish()));

    puffin_egui::puffin::profile_scope!("render present");
    output.present();

    *rendering_recorder_2.get_mut() = None;
}
