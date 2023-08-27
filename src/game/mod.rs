use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

pub mod opaque_pass;
pub mod shadow_pass;

use crate::{
    app::{Res, ResMut},
    asset_server::AssetServer,
    rendering::Renderer,
    scene::{scene_object::SceneObject, Scene},
};

use self::{opaque_pass::OpaqueRenderPass, shadow_pass::ShadowRenderPass};

#[derive(Debug, Clone, Copy)]
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

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: Mat4,
}

impl CameraUniform {
    fn update_view_projection(&mut self, camera_scene_object: &SceneObject) {
        self.view_projection = camera_scene_object
            .camera_component
            .as_ref()
            .unwrap()
            .build_view_projection_matrix(&camera_scene_object.transform_component);
    }
}

pub struct Game {
    // Move this to high level renderer
    pub sun: Sun,
    sun_uniform: SunUniform,
    pub sun_buffer: wgpu::Buffer,

    camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,

    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,

    pub shadow_pass: ShadowRenderPass,
    pub opaque_pass: OpaqueRenderPass,
}

impl Game {
    pub fn new(renderer: &mut Renderer) -> Self {
        let sun = Sun {
            inverse_direction: Vec3::new(4.0, 5.0, 1.0),
            projection: Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 100.0, 0.1),
        };

        let sun_uniform = SunUniform::default();

        let sun_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shadow pass bind group sun buffer"),
                contents: bytemuck::cast_slice(&[sun_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

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

        let shadow_pass = ShadowRenderPass::new(renderer, &sun_buffer);
        let opaque_pass = OpaqueRenderPass::new(renderer, &global_bind_group_layout, &sun_buffer);

        Self {
            sun,
            sun_uniform,
            sun_buffer,
            camera_uniform,
            camera_buffer,
            global_bind_group_layout,
            global_bind_group,
            shadow_pass,
            opaque_pass,
        }
    }
}

pub fn update(
    game: ResMut<Game>,
    scene: Res<Scene>,
    renderer: ResMut<Renderer>,
    asset_server: Res<AssetServer>,
) {
    let asset_server = asset_server.get();
    let mut app = game.get_mut();
    let mut renderer = renderer.get_mut();
    let scene = scene.get();

    renderer.create_render_meshes(&asset_server);
    renderer.create_render_materials(&asset_server);

    if let Some(camera_scene_object) = scene.get(scene.camera_scene_object_id) {
        app.camera_uniform
            .update_view_projection(&camera_scene_object);

        renderer.queue.write_buffer(
            &app.camera_buffer,
            0,
            bytemuck::cast_slice(&[app.camera_uniform]),
        );
    }

    let sun = app.sun.clone();
    app.sun_uniform.update(&sun);

    renderer
        .queue
        .write_buffer(&app.sun_buffer, 0, bytemuck::cast_slice(&[app.sun_uniform]));
}
