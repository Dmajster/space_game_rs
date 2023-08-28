use glam::{Mat4, Vec3, Vec4};
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

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SunUniform {
    mvp: Mat4,
    world_position: Vec4,
}

impl SunUniform {
    pub fn update(&mut self, sun_scene_object: &SceneObject) {
        let transform = &sun_scene_object.transform_component;
        let projection = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 100.0, 0.1);

        let view = Mat4::look_at_rh(transform.position, Vec3::ZERO, Vec3::Y);
        self.mvp = projection * view;
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: Mat4,
    pub world_position: Vec4,
}

impl CameraUniform {
    fn update(&mut self, camera_scene_object: &SceneObject) {
        self.view_projection = camera_scene_object
            .camera_component
            .as_ref()
            .unwrap()
            .build_view_projection_matrix(&camera_scene_object.transform_component);

        let position = camera_scene_object.transform_component.position;

        self.world_position = Vec4::new(position.x, position.y, position.z, 1.0);
    }
}

pub struct Game {
    // Move this to high level renderer
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
        app.camera_uniform.update(&camera_scene_object);

        renderer.queue.write_buffer(
            &app.camera_buffer,
            0,
            bytemuck::cast_slice(&[app.camera_uniform]),
        );
    }

    if let Some(camera_scene_object) = scene.get(scene.camera_scene_object_id) {
        app.camera_uniform.update(&camera_scene_object);

        renderer.queue.write_buffer(
            &app.camera_buffer,
            0,
            bytemuck::cast_slice(&[app.camera_uniform]),
        );
    }

    if let Some(sun_scene_object) = scene.get(scene.sun_scene_object_id) {
        app.sun_uniform.update(&sun_scene_object);

        renderer
            .queue
            .write_buffer(&app.sun_buffer, 0, bytemuck::cast_slice(&[app.sun_uniform]));
    }
}
