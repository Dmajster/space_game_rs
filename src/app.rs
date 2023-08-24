use std::{cell::RefCell, iter, rc::Rc};

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{
    asset_server::{AssetServer, self},
    egui_pass::EguiRenderPass,
    opaque_pass::OpaqueRenderPass,
    rendering::{RenderPass, Renderer},
    shadow_pass::ShadowRenderPass,
    Scene,
};

pub struct Sun {
    pub inverse_direction: Vec3,
    pub projection: Mat4,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunUniform {
    mvp: Mat4,
}

impl SunUniform {
    pub fn update(&mut self, sun: &Sun) {
        let view = Mat4::look_at_rh(sun.inverse_direction, Vec3::ZERO, Vec3::Y);
        self.mvp = sun.projection * view;
    }
}

pub struct Camera {
    pub transform: Mat4,
    pub projection: Mat4,
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

pub struct App<'app> {
    pub asset_server: AssetServer,
    pub renderer: Renderer<'app>,

    // Move this to high level renderer
    pub sun: Sun,
    sun_uniform: SunUniform,
    pub sun_buffer: wgpu::Buffer,

    pub camera: Camera,
    camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,

    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,

    pub passes: Option<Rc<RefCell<Passes>>>,
    pub scene: Scene,
}

pub struct Passes {
    pub shadow_pass: ShadowRenderPass,
    pub opaque_pass: OpaqueRenderPass,
    pub egui_pass: EguiRenderPass,
}

impl Passes {
    pub fn new(app: &mut App) -> Self {
        let shadow_pass = ShadowRenderPass::new(app);
        let opaque_pass = OpaqueRenderPass::new(app);
        let egui_pass = EguiRenderPass::new(app);

        Passes {
            shadow_pass,
            opaque_pass,
            egui_pass,
        }
    }
}

impl App<'_> {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Space game")
            .with_maximized(true)
            .build(event_loop)
            .unwrap();

        let renderer = Renderer::new(window);

        let sun = Sun {
            inverse_direction: Vec3::new(4.0, 5.0, 1.0),
            projection: Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 20.0, 0.1),
        };

        let sun_uniform = SunUniform::default();

        let sun_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shadow pass bind group sun buffer"),
                contents: bytemuck::cast_slice(&[sun_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera = Camera::new(
            Mat4::look_at_rh(Vec3::new(0.0, 5.0, 5.0), Vec3::ZERO, Vec3::Y),
            Mat4::perspective_infinite_reverse_rh(
                90.0f32.to_radians(),
                renderer.window.inner_size().width as f32
                    / renderer.window.inner_size().height as f32,
                0.1,
            ),
        );

        let camera_uniform = CameraUniform::default();

        let camera_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("renderer bind group camera buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let global_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let global_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("renderer bind group"),
                layout: &global_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        let asset_server = AssetServer::read_from_file_or_new(&asset_server::DEFAULT_PATH);

        Self {
            asset_server,
            renderer,
            scene: Default::default(),
            sun,
            sun_uniform,
            sun_buffer,
            camera,
            camera_uniform,
            camera_buffer,
            global_bind_group_layout,
            global_bind_group,
            passes: None,
        }
    }

    pub fn pre_update(&mut self) {
        self.renderer.create_render_meshes(&self.asset_server);

        self.camera_uniform.update_view_projection(&self.camera);

        self.renderer.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.sun_uniform.update(&self.sun);

        self.renderer.queue.write_buffer(
            &self.sun_buffer,
            0,
            bytemuck::cast_slice(&[self.sun_uniform]),
        );

        let instances = self
            .scene
            .scene_object_hierarchy
            .scene_objects
            .iter()
            .map(|mut scene_object| {
                let mut transform = scene_object.transform;

                loop {
                    if let Some(parent_id) = scene_object.parent {
                        let parent = self
                            .scene
                            .scene_object_hierarchy
                            .scene_objects
                            .iter()
                            .find(|scene_object| scene_object.id == parent_id)
                            .unwrap();

                        transform *= parent.transform;

                        scene_object = parent;
                    } else {
                        break;
                    }
                }

                transform
            })
            .collect::<Vec<_>>();

        self.renderer.queue.write_buffer(
            &self.renderer.scene_object_instances,
            0,
            bytemuck::cast_slice(instances.as_slice()),
        );

        let mut passes = self.passes.as_ref().unwrap().borrow_mut();
        passes.shadow_pass.prepare(&self);
        passes.opaque_pass.prepare(&self);
        passes.egui_pass.prepare(&self);
    }

    pub fn update(&mut self) {
        let output = self.renderer.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let mut passes = self.passes.as_ref().unwrap().borrow_mut();
        passes.shadow_pass.render(&self, &mut encoder, &view);
        passes.opaque_pass.render(&self, &mut encoder, &view);
        passes.egui_pass.render(&self, &mut encoder, &view);

        self.renderer.queue.submit(iter::once(encoder.finish()));
        output.present();
    }

    pub fn post_update(&mut self) {
        let mut passes = self.passes.as_ref().unwrap().borrow_mut();
        passes.shadow_pass.cleanup(&self);
        passes.opaque_pass.cleanup(&self);
        passes.egui_pass.cleanup(&self);
    }

    pub fn close(&mut self) {
        self.asset_server.write_to_file(&asset_server::DEFAULT_PATH)
    }
}
