use winit::window::Window;

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

    // RENDER PASSES
    // shadow_render_pass: wgpu::RenderPass<'renderer>,
    // z_prepass_render_pass: wgpu::RenderPass<'renderer>,


    
    // RENDER OBJECT CACHING. Has no reclaiming so it grows infinitely with each new variant...
    // Might be a problem. Reclaiming would require tracking object usage.
    // bind_group_cache: BTreeMap<BindGroupDescriptor<'renderer>, BindGroup>,
    // bind_group_layout_cache: BTreeMap<BindGroupLayoutDescriptor<'renderer>, BindGroupLayout>,
    pipeline_layout_descriptors_cache: Vec<wgpu::PipelineLayoutDescriptor<'renderer>>,
    pipeline_layouts_cache: Vec<wgpu::PipelineLayout>,
    render_pipeline_descriptors_cache: Vec<wgpu::RenderPipelineDescriptor<'renderer>>,
    render_pipelines_cache: Vec<wgpu::RenderPipeline>,
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

            // bind_group_cache: Default::default(),
            // bind_group_layout_cache: Default::default(),
            pipeline_layout_descriptors_cache: Default::default(),
            pipeline_layouts_cache: Default::default(),
            render_pipelines_cache: Default::default(),
            render_pipeline_descriptors_cache: Default::default(),
        }
    }

    pub fn get_pipeline_layout(
        &mut self,
        descriptor: &wgpu::PipelineLayoutDescriptor<'renderer>,
    ) -> &wgpu::PipelineLayout {
        let mut matching_layout_index = usize::MAX;

        // Check if we already have a matching pipeline
        for (i, compared_descriptor) in self.pipeline_layout_descriptors_cache.iter().enumerate() {
            if Renderer::are_pipeline_layout_descriptors_equal(descriptor, compared_descriptor)
                && descriptor.push_constant_ranges == compared_descriptor.push_constant_ranges
            {
                matching_layout_index = i;
            }
        }

        // Create a new pipeline if none match
        if matching_layout_index == usize::MAX {
            let new_pipeline_layout = self.device.create_pipeline_layout(&descriptor);

            matching_layout_index = self.render_pipelines_cache.len();
            self.pipeline_layouts_cache.push(new_pipeline_layout);
            self.pipeline_layout_descriptors_cache
                .push(descriptor.clone());
        }

        &self.pipeline_layouts_cache[matching_layout_index]
    }

    fn are_pipeline_layout_descriptors_equal(
        a: &wgpu::PipelineLayoutDescriptor<'_>,
        b: &wgpu::PipelineLayoutDescriptor<'_>,
    ) -> bool {
        if a.bind_group_layouts.len() != b.bind_group_layouts.len() {
            return false;
        }

        for (a_bind_group_layout, b_bind_group_layout) in
            a.bind_group_layouts.iter().zip(b.bind_group_layouts.iter())
        {
            if a_bind_group_layout.global_id() != b_bind_group_layout.global_id() {
                return false;
            }
        }

        if a.push_constant_ranges != b.push_constant_ranges {
            return false;
        }

        true
    }

    pub fn get_render_pipeline(
        &mut self,
        descriptor: &wgpu::RenderPipelineDescriptor<'renderer>,
    ) -> &wgpu::RenderPipeline {
        let mut matching_pipeline_index = usize::MAX;

        // Check if we already have a matching pipeline
        for (i, compared_descriptor) in self.render_pipeline_descriptors_cache.iter().enumerate() {
            if Renderer::are_render_pipeline_descriptors_equal(&descriptor, compared_descriptor) {
                matching_pipeline_index = i;
            }
        }

        // Create a new pipeline if none match
        if matching_pipeline_index == usize::MAX {
            let new_pipeline = self.device.create_render_pipeline(&descriptor);

            matching_pipeline_index = self.render_pipelines_cache.len();
            self.render_pipelines_cache.push(new_pipeline);
            self.render_pipeline_descriptors_cache
                .push(descriptor.clone());
        }

        &self.render_pipelines_cache[matching_pipeline_index]
    }

    fn are_render_pipeline_descriptors_equal(
        a: &wgpu::RenderPipelineDescriptor<'_>,
        b: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> bool {
        let layouts_match = {
            if a.layout.is_some() && b.layout.is_some() {
                a.layout.unwrap().global_id() == b.layout.unwrap().global_id()
            } else if a.layout.is_none() && b.layout.is_none() {
                true
            } else {
                false
            }
        };

        let vertices_match = {
            a.vertex.module.global_id() == b.vertex.module.global_id()
                && a.vertex.entry_point == b.vertex.entry_point
                && a.vertex.buffers == b.vertex.buffers
        };

        let fragments_match = {
            if a.fragment.is_some() && b.fragment.is_some() {
                let fragment = a.fragment.as_ref().unwrap();
                let b_fragment = b.fragment.as_ref().unwrap();

                fragment.module.global_id() == b_fragment.module.global_id()
                    && fragment.entry_point == b_fragment.entry_point
                    && fragment.targets == b_fragment.targets
            } else if a.fragment.is_none() && b.fragment.is_none() {
                true
            } else {
                false
            }
        };

        layouts_match
            && vertices_match
            && fragments_match
            && a.depth_stencil == b.depth_stencil
            && a.primitive == b.primitive
            && a.multisample == b.multisample
            && a.multiview == b.multiview
    }
}

pub struct Material<'material> {
    pub pipeline_layout_descriptor: wgpu::RenderPipelineDescriptor<'material>,
}
