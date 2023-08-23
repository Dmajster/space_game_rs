use wgpu::{CommandEncoder, TextureView};

use crate::App;

pub trait RenderPass {
    fn prepare(&mut self, app: &App);

    fn render(&mut self, app: &App, encoder: &mut CommandEncoder, view: &TextureView);

    fn cleanup(&mut self, app: &App);
}

use egui::epaint::ahash::HashMap;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::{collections::BTreeMap, marker::PhantomData};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::Mesh;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderInstance {
    pub model_matrix_0: Vec4,
    pub model_matrix_1: Vec4,
    pub model_matrix_2: Vec4,
    pub model_matrix_3: Vec4,
}

pub struct RenderSceneObject {
    pub transform: Mat4, // Optimize to Mat4x3
    pub mesh_handle: Handle<RenderMesh>,
}

pub struct RenderMesh {
    pub vertex_buffer_handle: Handle<wgpu::Buffer>,
    pub vertex_offset: usize,
    pub vertex_count: usize,
    pub index_buffer_handle: Handle<wgpu::Buffer>,
    pub index_offset: usize,
    pub index_count: usize,
}

#[derive(Debug)]
pub struct Handle<T> {
    pub index: usize,
    pub generation: usize,
    _pd: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            generation: self.generation.clone(),
            _pd: self._pd.clone(),
        }
    }
}

impl<T> Copy for Handle<T> {}

#[derive(Debug)]
pub struct Pool<T> {
    objects: Vec<T>,
    generations: Vec<usize>,
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self {
            objects: Default::default(),
            generations: Default::default(),
        }
    }
}

impl<T> Pool<T> {
    pub fn add(&mut self, object: T) -> Handle<T> {
        let index = self.objects.len();
        self.objects.push(object);
        self.generations.push(0);

        Handle {
            index,
            generation: 0,
            _pd: PhantomData,
        }
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        if handle.index < self.objects.len() && handle.generation == self.generations[handle.index]
        {
            Some(&self.objects[handle.index])
        } else {
            None
        }
    }
}

pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const SCENE_OBJECT_INSTANCES_BUFFER_SIZE: u64 = 20 * 1024 * 1024; //20MB

pub struct Renderer<'renderer> {
    pub window: Window,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub surface_capabilities: wgpu::SurfaceCapabilities,
    pub surface_format: wgpu::TextureFormat,
    pub surface_configuration: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub render_pass_resources: HashMap<&'renderer str, wgpu::Texture>,

    pub scene_objects: Vec<RenderSceneObject>,
    pub scene_object_instances: wgpu::Buffer,
    pub meshes: Pool<RenderMesh>,
    pub mesh_buffers: Pool<wgpu::Buffer>,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

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
    pub fn new(window: Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
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

        Self {
            window,
            surface,
            surface_capabilities,
            surface_format,
            surface_configuration,
            instance,
            adapter,
            device,
            queue,

            render_pass_resources: Default::default(),

            scene_objects: Default::default(),

            depth_texture,
            depth_texture_view,
            sampler,

            meshes: Default::default(),
            mesh_buffers: Default::default(),

            _bind_group_cache: Default::default(),
            _bind_group_layout_cache: Default::default(),
            _pipeline_layout_cache: Default::default(),
            _render_pipeline_cache: Default::default(),
            scene_object_instances,
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<RenderMesh> {
        let vertex_data = mesh
            .positions
            .into_iter()
            .zip(mesh.normals.into_iter())
            .zip(mesh.uvs.into_iter())
            .map(|((position, normal), uv)| Vertex {
                position,
                normal,
                uv,
            })
            .collect::<Vec<_>>();

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

        self.meshes.add(RenderMesh {
            vertex_buffer_handle,
            vertex_offset: 0,
            vertex_count: vertex_data.len(),
            index_buffer_handle,
            index_offset: 0,
            index_count: mesh.indices.len(),
        })
    }
}
