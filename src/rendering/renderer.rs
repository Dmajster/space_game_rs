use glam::{Mat4, Vec2, Vec3};
use std::{cell::RefCell, collections::BTreeMap, iter, rc::Rc};
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::RenderPass;

pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct Camera {
    pub transform: Mat4,
    pub projection: Mat4,
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

impl Camera {
    fn new(transform: Mat4, projection: Mat4) -> Self {
        Self {
            transform,
            projection,
        }
    }

    fn build_view_projection_matrix(&self) -> Mat4 {
        self.projection * self.transform
    }
}

pub struct RenderObject {
    pub transform: Mat4, // Optimize to Mat4x3
    pub mesh_index: usize,
    pub material_index: usize,
}

pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,

    render_objects: Vec<RenderObject>,

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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera = Camera::new(
            Mat4::look_at_rh(Vec3::new(10.0, 0.0, 0.0), Vec3::ZERO, Vec3::Y),
            Mat4::perspective_infinite_reverse_rh(
                90.0f32.to_radians(),
                window.inner_size().width as f32 / window.inner_size().height as f32,
                0.1,
            ),
        );

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_projection(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("renderer bind group camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("renderer bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
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

            render_objects: Default::default(),
            camera,
            camera_uniform,
            camera_buffer,

            bind_group_layout,
            bind_group,

            bind_group_cache: Default::default(),
            bind_group_layout_cache: Default::default(),
            pipeline_layout_cache: Default::default(),
            render_pipeline_cache: Default::default(),
            render_passes: Default::default(),
        }
    }

    pub fn add_pass<P>(&mut self, pass: P)
    where
        P: RenderPass + 'static,
    {
        self.render_passes.push(Rc::new(RefCell::new(pass)));
    }

    pub fn prepare(&mut self) {
        self.camera_uniform.update_view_projection(&self.camera);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
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
}
