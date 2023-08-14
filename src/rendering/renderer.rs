use std::collections::BTreeMap;

use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, Device,
    Instance, PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceCapabilities, SurfaceConfiguration, TextureFormat,
};
use winit::window::Window;

pub struct Renderer<'renderer> {
    pub window: Window,
    pub instance: Instance,
    pub surface: Surface,
    pub surface_capabilities: SurfaceCapabilities,
    pub surface_format: TextureFormat,
    pub surface_configuration: SurfaceConfiguration,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,

    // RENDER OBJECT CACHING. Has no reclaiming so it grows infinitely with each new variant...
    // Might be a problem. Reclaiming would require tracking object usage.
    bind_group_cache: BTreeMap<BindGroupDescriptor<'renderer>, BindGroup>,
    bind_group_layout_cache: BTreeMap<BindGroupLayoutDescriptor<'renderer>, BindGroupLayout>,
    pipeline_layout_cache: BTreeMap<PipelineLayoutDescriptor<'renderer>, PipelineLayout>,
    render_pipeline_cache: BTreeMap<RenderPipelineDescriptor<'renderer>, RenderPipeline>,
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

            bind_group_cache: Default::default(),
            bind_group_layout_cache: Default::default(),
            pipeline_layout_cache: Default::default(),
            render_pipeline_cache: Default::default(),
        }
    }
}
