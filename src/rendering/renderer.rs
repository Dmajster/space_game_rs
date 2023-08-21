use egui::epaint::ahash::HashMap;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::{cell::RefCell, collections::BTreeMap, iter, marker::PhantomData, rc::Rc};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{Camera, Mesh, Sun};

use super::{shadow_pass::SunUniform, RenderPass};

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

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: Mat4,
}

impl CameraUniform {
    fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix();
    }
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

    // Everything below this should be in it's own struct and not part of the Renderer base
    pub render_passes: Vec<Rc<RefCell<dyn RenderPass>>>,
    pub render_pass_resources: HashMap<&'renderer str, wgpu::Texture>,

    pub sun: Sun,
    sun_uniform: SunUniform,
    pub sun_buffer: wgpu::Buffer,

    pub camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,

    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,

    pub scene_objects: Vec<RenderSceneObject>,
    pub scene_object_instances: wgpu::Buffer,

    pub meshes: Pool<RenderMesh>,
    pub mesh_buffers: Pool<wgpu::Buffer>,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,

    pub sampler: wgpu::Sampler,

    // RENDER PASSES
    // shadow_render_pass: wgpu::RenderPass<'renderer>,
    // z_prepass_render_pass: wgpu::RenderPass<'renderer>,


    
    // RENDER OBJECT CACHING. Has no reclaiming so it grows infinitely with each new variant...
    // Might be a problem. Reclaiming would require tracking object usage.
    bind_group_cache: BTreeMap<wgpu::BindGroupDescriptor<'renderer>, wgpu::BindGroup>,
    bind_group_layout_cache:
        BTreeMap<wgpu::BindGroupLayoutDescriptor<'renderer>, wgpu::BindGroupLayout>,
    pipeline_layout_cache:
        BTreeMap<wgpu::PipelineLayoutDescriptor<'renderer>, wgpu::PipelineLayout>,
    render_pipeline_cache:
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

        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("renderer bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

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
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sun = Sun {
            inverse_direction: Vec3::new(4.0, 5.0, 1.0),
            projection: Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 20.0, 0.1),
        };

        let sun_uniform = SunUniform::default();

        let sun_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow pass bind group sun buffer"),
            contents: bytemuck::cast_slice(&[sun_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera = Camera::new(
            Mat4::look_at_rh(Vec3::new(0.0, 5.0, 5.0), Vec3::ZERO, Vec3::Y),
            Mat4::perspective_infinite_reverse_rh(
                90.0f32.to_radians(),
                window.inner_size().width as f32 / window.inner_size().height as f32,
                0.1,
            ),
        );

        let camera_uniform = CameraUniform::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("renderer bind group camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("renderer bind group"),
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
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

            render_passes: Default::default(),
            render_pass_resources: Default::default(),

            scene_objects: Default::default(),

            depth_texture,
            depth_texture_view,
            sampler,

            sun,
            sun_uniform,
            sun_buffer,

            camera,
            camera_uniform,
            camera_buffer,

            global_bind_group_layout,
            global_bind_group,

            meshes: Default::default(),
            mesh_buffers: Default::default(),

            bind_group_cache: Default::default(),
            bind_group_layout_cache: Default::default(),
            pipeline_layout_cache: Default::default(),
            render_pipeline_cache: Default::default(),
            scene_object_instances,
        }
    }

    pub fn prepare(&mut self) {
        self.camera_uniform.update_view_projection(&self.camera);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.sun_uniform.update(&self.camera, &self.sun);

        self.queue.write_buffer(
            &self.sun_buffer,
            0,
            bytemuck::cast_slice(&[self.sun_uniform]),
        );

        let instances = self
            .scene_objects
            .iter()
            .map(|scene_object| scene_object.transform)
            .collect::<Vec<_>>();

        self.queue.write_buffer(
            &self.scene_object_instances,
            0,
            bytemuck::cast_slice(instances.as_slice()),
        );

        for pass in &self.render_passes {
            pass.borrow_mut().prepare(self)
        }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        for pass in &self.render_passes {
            pass.borrow_mut().render(self, &mut encoder, &view);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
    }

    pub fn cleanup(&mut self) {
        for pass in &self.render_passes {
            pass.borrow_mut().cleanup(self)
        }
    }

    pub fn add_pass<P>(&mut self, pass: P)
    where
        P: RenderPass + 'static,
    {
        self.render_passes.push(Rc::new(RefCell::new(pass)));
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
